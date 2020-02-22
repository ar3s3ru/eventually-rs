use std::fmt::Debug;
use std::{error::Error as StdError, future::Future};

use futures::stream::{StreamExt, TryStream, TryStreamExt};

use eventually_core::store::Store as EventStore;
use eventually_core::{aggregate, aggregate::AggregateExt};
use eventually_core::{command, command::Handler as CommandHandler};

pub type SourceIdOf<I: Identifiable> = I::SourceId;

pub trait Identifiable {
    type SourceId: Eq;

    fn source_id(&self) -> Self::SourceId;
}

#[derive(Debug, PartialEq)]
pub enum Error<A, C, S> {
    RecreateStateFailed(A),
    CommandFailed(C),
    ApplyStateFailed(A),
    AppendEventsFailed(S),
}

impl<A, C, S> StdError for Error<A, C, S>
where
    A: StdError + 'static,
    C: StdError + 'static,
    S: StdError + 'static,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(match self {
            Error::RecreateStateFailed(e) => e,
            Error::CommandFailed(e) => e,
            Error::ApplyStateFailed(e) => e,
            Error::AppendEventsFailed(e) => e,
        })
    }
}

impl<A, C, RS> std::fmt::Display for Error<A, C, RS>
where
    A: StdError,
    C: StdError,
    RS: StdError,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::RecreateStateFailed(e) => write!(f, "Failed to recreate state: {}", e),
            Error::CommandFailed(e) => write!(f, "Failed to handle command: {}", e),
            Error::ApplyStateFailed(e) => write!(f, "Failed to apply new events: {}", e),
            Error::AppendEventsFailed(e) => write!(f, "Failed to append events to store: {}", e),
        }
    }
}

pub struct Dispatcher<Store, Handler> {
    store: Store,
    handler: Handler,
}

impl<Store, Handler> Dispatcher<Store, Handler>
where
    Handler: CommandHandler + Send,
    Store: EventStore + Send,
    <Store as EventStore>::SourceId: Clone + Eq + Send,
    <Store as EventStore>::Offset: Default + Send,
    <Store as EventStore>::Stream: TryStream + Send,
    <Store as EventStore>::Error: StdError + Send + 'static,
    <<Store as EventStore>::Stream as TryStream>::Error: Debug,
    command::AggregateOf<Handler>: AggregateExt<Event = <Store as EventStore>::Event> + Send,
    command::CommandOf<Handler>: Identifiable<SourceId = <Store as EventStore>::SourceId> + Send,
    aggregate::EventOf<command::AggregateOf<Handler>>: Clone + Send,
    aggregate::StateOf<command::AggregateOf<Handler>>: Default + Send,
    aggregate::ErrorOf<command::AggregateOf<Handler>>: StdError + Send + 'static,
    command::ErrorOf<Handler>: StdError + Send + 'static,
{
    #[inline]
    pub fn new(store: Store, handler: Handler) -> Self {
        Dispatcher { store, handler }
    }

    pub fn dispatch(
        &mut self,
        c: command::CommandOf<Handler>,
    ) -> impl Future<
        Output = Result<
            aggregate::StateOf<command::AggregateOf<Handler>>,
            Error<aggregate::ErrorOf<command::AggregateOf<Handler>>, Handler::Error, Store::Error>,
        >,
    > + '_ {
        async move {
            let id = c.source_id();

            let events = self
                .store
                .stream(id.clone(), <Store as EventStore>::Offset::default());

            let state = command::AggregateOf::<Handler>::async_fold(
                aggregate::StateOf::<command::AggregateOf<Handler>>::default(),
                // TODO: remove this unwrap and do some proper error handling
                events.into_stream().map(|result| result.unwrap()),
            )
            .await
            .map_err(Error::RecreateStateFailed)?;

            let new_events = self
                .handler
                .handle(&state, c)
                .await
                .map_err(Error::CommandFailed)?;

            let new_state =
                command::AggregateOf::<Handler>::fold(state, new_events.clone().into_iter())
                    .map_err(Error::ApplyStateFailed)?;

            self.store
                .append(id, new_events)
                .await
                .map_err(Error::AppendEventsFailed)?;

            Ok(new_state)
        }
    }
}
