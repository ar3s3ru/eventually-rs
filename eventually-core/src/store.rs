//! Contains the Event Store trait for storing and streaming Aggregate [`Event`]s.
//!
//! [`Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event

use futures::future::BoxFuture;
use futures::stream::BoxStream;

/// An Event Store is an append-only, ordered list of [`Event`]s
/// for a certain "source" -- e.g. an [`Aggregate`].
///
/// [`Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event
/// [`Aggregate`]: ../aggregate/trait.Aggregate.html
pub trait EventStore {
    /// Type of the Source id, typically an [`AggregateId`].
    ///
    /// [`AggregateId`]: ../aggregate/type.AggregateId.html
    type SourceId: Eq;

    /// Offset type for getting a slice of the [`Event`]s in the Store.
    ///
    /// Check out [`stream`] for more info.
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`stream`]: trait.EventStore.html#method.stream
    type Offset: Ord;

    /// Event to be stored in the `EventStore`, typically an [`Aggregate::Event`].
    ///
    /// [`Aggregate::Event`]: ../aggregate/trait.Aggregate.html#associatedtype.Event
    type Event;

    /// Possible errors returned by the `EventStore` when requesting operations.
    type Error;

    /// Appends a new list of [`Event`]s to the Event Store, for the Source
    /// entity specified by [`SourceId`].
    ///
    /// `append` is a transactional operation: it either appends all the events,
    /// or none at all and returns an appropriate [`Error`].
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`SourceId`]: trait.EventStore.html#associatedtype.SourceId
    /// [`Error`]: trait.EventStore.html#associatedtype.Error
    fn append(
        &mut self,
        id: Self::SourceId,
        events: Vec<Self::Event>,
    ) -> BoxFuture<Result<(), Self::Error>>;

    /// Streams a list of [`Event`]s from the `EventStore` back to the application,
    /// by specifying the desired [`SourceId`] and [`Offset`].
    ///
    /// [`SourceId`] will be used to request a particular `EventStream`.
    ///
    /// [`Offset`] will be used to specify a slice of the [`Event`]s to retrieve
    /// from the `EventStore`. To request the whole list, use the [`Default`]
    /// value for [`Offset`].
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`SourceId`]: trait.EventStore.html#associatedtype.SourceId
    /// [`Offset`]: trait.EventStore.html#associatedtype.Offset
    fn stream(
        &self,
        id: Self::SourceId,
        from: Self::Offset,
    ) -> BoxFuture<Result<BoxStream<Result<Self::Event, Self::Error>>, Self::Error>>;

    /// Drops all the [`Event`]s related to one `Source`, specified by
    /// the provided [`SourceId`].
    ///
    /// [`Event`]: trait.EventStore.html#associatedtype.Event
    /// [`SourceId`]: trait.EventStore.html#associatedtype.SourceId
    fn remove(&mut self, id: Self::SourceId) -> BoxFuture<Result<(), Self::Error>>;
}