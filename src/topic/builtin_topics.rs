pub mod ros_discovery {
    use rustdds::{
        policy::{Deadline, Durability, History, Lifespan, Ownership},
        QosPolicies, QosPolicyBuilder,
    };

    lazy_static! {
        pub static ref QOS_PUB: QosPolicies = QosPolicyBuilder::new()
            .durability(Durability::TransientLocal)
            .deadline(Deadline(rustdds::Duration::INFINITE))
            .ownership(Ownership::Shared)
            .reliable(rustdds::Duration::ZERO)
            .history(History::KeepLast { depth: 1 })
            .lifespan(Lifespan {
                duration: rustdds::Duration::INFINITE
            })
            .build();
        pub static ref QOS_SUB: QosPolicies = QosPolicyBuilder::new()
            .durability(Durability::Volatile)
            .ownership(Ownership::Shared)
            .reliable(rustdds::Duration::ZERO)
            .history(History::KeepLast { depth: 1 })
            .lifespan(Lifespan {
                duration: rustdds::Duration::INFINITE
            })
            .build();
    }

    pub const TOPIC_NAME: &str = "ros_discovery_info";

    pub const TYPE_NAME: &str = "rmw_dds_common::msg::dds_::ParticipantEntitiesInfo_";
}

pub mod parameter_events {
    use rustdds::{
        policy::{Durability, History},
        QosPolicies, QosPolicyBuilder,
    };

    lazy_static! {
        pub static ref QOS: QosPolicies = QosPolicyBuilder::new()
            .durability(Durability::TransientLocal)
            .reliable(rustdds::Duration::ZERO)
            .history(History::KeepLast { depth: 1 })
            .build();
    }

    pub const TOPIC_NAME: &str = "rt/parameter_events";

    pub const TYPE_NAME: &str = "rcl_interfaces::msg::dds_::ParameterEvent_";
}

pub mod rosout {
    use rustdds::{
        policy::{Deadline, Durability, History, Lifespan, Ownership},
        QosPolicies, QosPolicyBuilder,
    };

    lazy_static! {
        pub static ref QOS: QosPolicies = QosPolicyBuilder::new()
            .durability(Durability::TransientLocal)
            .deadline(Deadline(rustdds::Duration::INFINITE))
            .ownership(Ownership::Shared)
            .reliable(rustdds::Duration::ZERO)
            .history(History::KeepLast { depth: 1 })
            .lifespan(Lifespan {
                duration: rustdds::Duration::from_secs(10)
            })
            .build();
    }

    pub const TOPIC_NAME: &str = "rt/rosout";

    pub const TYPE_NAME: &str = "rcl_interfaces::msg::dds_::Log_";
}
