//! ROS 2 client library, similar to the [rclcpp](https://docs.ros.org/en/rolling/p/rclcpp/) or
//! [rclpy](https://docs.ros.org/en/rolling/p/rclpy/) libraries, in native Rust. The underlying DDS
//! implementation, [RustDDS](https://atostek.com/en/products/rustdds/), is also native Rust.
//!
//! # Example
//!
//! ```
//! use futures::StreamExt;
//! use ros2_client::*;
//!
//!   let context = Context::new().unwrap();
//!   let mut node = context
//!     .new_node(
//!       NodeName::new("/rustdds", "rustdds_listener").unwrap(),
//!       NodeOptions::new().enable_rosout(true),
//!     )
//!     .unwrap();
//!
//!   let chatter_topic = node
//!     .create_topic(
//!       &Name::new("/","topic").unwrap(),
//!       MessageTypeName::new("std_msgs", "String"),
//!       &ros2_client::DEFAULT_SUBSCRIPTION_QOS,
//!     )
//!     .unwrap();
//!   let chatter_subscription = node
//!     .create_subscription::<String>(&chatter_topic, None)
//!     .unwrap();
//!
//!   let subscription_stream = chatter_subscription
//!     .async_stream()
//!     .for_each(|result| async {
//!       match result {
//!         Ok((msg, _)) => println!("I heard: {msg}"),
//!         Err(e) => eprintln!("Receive request error: {:?}", e),
//!       }
//!     });
//!
//!   // Since we enabled rosout, let's log something
//!   rosout!(
//!     node,
//!     ros2::LogLevel::Info,
//!     "wow. very listening. such topics. much subscribe."
//!   );
//!
//!   // Uncomment this to execute until interrupted.
//!   // --> smol::block_on( subscription_stream );
//! ```

pub mod action;
pub mod interfaces;
pub mod log;
pub mod message;
pub mod node;
pub mod service;
pub mod time;
pub mod topic;

/// Common types in this crate.
pub mod prelude {
    pub use crate::action::{Action, ActionTypes};
    pub use crate::message::{message_info::MessageInfo, Message};
    pub use crate::topic::Topic;

    pub use crate::interfaces::{
        names::{ActionTypeName, MessageTypeName, Name, NodeName, ServiceTypeName},
        rcl_interfaces::*,
        wide_string::WString,
    };

    pub use crate::service::{
        client::CallServiceError,
        client::Client,
        parameters::{Parameter, ParameterValue},
        server::Server,
        AService, Service, ServiceMapping,
    };

    pub use crate::node::{
        context::{Context, ContextOptions, DEFAULT_PUBLISHER_QOS, DEFAULT_SUBSCRIPTION_QOS},
        pubsub::{Publisher, Subscription},
        Node, NodeCreateError, NodeEvent, NodeOptions, Spinner,
    };

    // time
    pub use crate::time::{ros_time::ROSTime, ros_time::SystemTime};

    // logging
    pub use crate::log::{Log, LogLevel};
    pub use crate::rosout;

    /// Stuff related to the [`rustdds`] crate. In a separate prelude since
    /// many users won't need these types.
    pub mod dds {
        pub use rustdds::{
            dds::WriteError,
            policy::{Deadline, Durability, History, Lifespan, Liveliness, Reliability},
            Duration as DdsDuration, QosPolicies, QosPolicyBuilder, Timestamp,
        };
    }
}
