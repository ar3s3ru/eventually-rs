use std::sync::Arc;

use eventually::aggregate::Versioned;
use eventually::optional::AsAggregate as Optional;
use eventually::versioned::Versioned as VersionedData;
use eventually::Repository;
use eventually_postgres::EventStore;

// use avento_postgres::EventStore;

use tokio::sync::RwLock;

use crate::order;

pub(crate) type OrderAggregate = Arc<Versioned<Optional<order::OrderAggregate>>>;
pub(crate) type OrderStore = EventStore<String, VersionedData<order::OrderEvent>>;
// pub(crate) type OrderStore = InMemoryStore<String, VersionedData<order::OrderEvent>>;
pub(crate) type OrderRepository = Repository<OrderAggregate, OrderStore>;

pub(crate) struct AppState {
    pub store: OrderStore,
    pub aggregate: OrderAggregate,
    pub repository: Arc<RwLock<OrderRepository>>,
}
