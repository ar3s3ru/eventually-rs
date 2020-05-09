//! Contains abstractions for the Event Store feature.

use futures::future::BoxFuture;
use futures::stream::BoxStream;

/// Represents an Event Store, an append-only, ordered list of [`Events`]
/// for a certain "source" (i.e. [`Aggregate`]).
///
/// [`Events`]: trait.Store.html#associatedType.Event
/// [`Aggregate`]: ../aggregate/trait.Aggregate.html
pub trait Store {
    /// Type of the Source id, usually an [`Aggregate`] id.
    ///
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    type SourceId: PartialEq;

    /// Type of the memory offset supported by the `Store`.
    ///
    /// An offset is needed to get a slice of the events in the `Store`.
    ///
    /// Check out [`Store::stream`] for more information.
    ///
    /// [`Store::stream`]: trait.Store.html#method.stream
    type Offset: PartialOrd;

    /// Type of the events supported by the `Store`.
    ///
    /// Usually, they match the same type as [`Aggregate::Event`].
    ///
    /// [`Aggregate::Event`]: ../aggregate/trait.Aggregate.html#associatedType.Event
    type Event;

    /// Possible errors returned by the `Store`.
    ///
    /// Usually, this should be an `enum` containing all the possible reasons
    /// why the `Store` could fail due to external failures.
    ///
    /// [`append`]: trait.Store.html#method.append
    type Error;

    /// Allows to stream many [`Events`] from the `Store` back to the application,
    /// by specifying the [`SourceId`] and the desired [`Offset`].
    ///
    /// An asynchronous [`Stream`] is returned, yielding every `Event`
    /// from the specified `Offset` in the same order as they were committed.
    ///
    /// To stream back all the events in the `Store` for a certain `Source`,
    /// [`Offset`] should implement `Default`, and use such value (in case of numeric offsets,
    /// this is most likely `0`).
    ///
    /// In case of an [`Aggregate`], it is possible to recreate its [`State`] by
    /// supplying the returned [`Stream`] into [`Aggregate::async_fold`].
    ///
    /// [`Events`]: trait.Store.html#associatedType.Events
    /// [`SourceId`]: trait.Store.html#associatedType.SourceId
    /// [`Offset`]: trait.Store.html#associatedType.SourceId
    /// [`Stream`]: https://docs.rs/futures/stream/trait.Stream.html
    /// [`Aggregate`]: ../aggregate/trait.Aggregate.html
    /// [`State`]: ../aggregate/trait.Aggregate.html#associatedType.State
    /// [`Aggregate::async_fold`]: ../aggregate/trait.AggregateExt.html#method.async_fold
    fn stream(
        &self,
        source_id: Self::SourceId,
        from: Self::Offset,
    ) -> BoxStream<Result<Self::Event, Self::Error>>;

    /// Appends a list of new events to the `Store`.
    ///
    /// An [`Error`] is returned if the append operation fails.
    ///
    /// [`Error`]: trait.Store.html#associatedType.Error
    fn append(
        &mut self,
        source_id: Self::SourceId,
        events: Vec<Self::Event>,
    ) -> BoxFuture<Result<(), Self::Error>>;
}
