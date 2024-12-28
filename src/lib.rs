//! # `ros2_client`
//!
//! A ROS 2 client library for Rust. It's similar to [`rclcpp`](https://docs.ros.org/en/rolling/p/rclcpp/) and [`rclpy`](https://docs.ros.org/en/rolling/p/rclpy/), but made in native Rust.
//!
//! ## Quick Start
//!     
//! Most of this crate's types are readily available in its `prelude`. You can import it like so:
//!
//! ```
//! use ros2_client::prelude::*;
//! ```
//!
//! ### Example
//!
//! With the types in that prelude, we can now start building our Nodes and other ROS 2 types! Let's start by writing a simple chat subscriber node:
//!
//! ```
//! use ros2_client::prelude::{dds::*, *};
//!
//! # smol::block_on(doctest());
//! # async fn doctest() {
//! // In `ros2_client`, the `Context` struct needs to be initialized before you
//! // can make anything talk.
//! //
//! // That's because it holds the DDS layer, which handles all networking here.
//! let ctx = Context::from_domain_participant(DomainParticipant::new(1).expect("dds security"))
//!     .expect("there shouldn't be another context");
//!
//! // Let's make a node!
//! let mut node = {
//!     // We'll need a few things first:
//!     //
//!     // 1. `ctx` (done)
//!     // 2. NodeName
//!     // 3. NodeOptions
//!
//!     // I'll start with the name:
//!     let node_name = NodeName::new("/rustdds", "example_node")
//!         .expect("node name should follow ROS 2 naming conventions");
//!
//!     // Great! Now, we can focus on the options.
//!     //
//!     // Most of the time, we'll want to turn on `rosout`, which allows the node to
//!     // print into the ROS 2 domain's `rosout` topic.
//!     //
//!     // Or, in other words, we can print to a shared 'console'.
//!     let node_options = NodeOptions::new()
//!         .enable_rosout(true) // this is on by default but shh...
//!         .declare_parameter("is_cool", ParameterValue::Boolean(true));
//!
//!     // Finally, we can make the node:
//!     ctx.new_node(node_name, node_options)
//!         .expect("node creation should succeed")
//! };
//!
//! // Alright, our node is finished. Let's make a topic for it to subscribe to!
//! let topic: Topic = {
//!     // Topics require the following:
//!     //
//!     // 1. a Node to create it (done)
//!     // 2. some Name
//!     // 3. A known Message type
//!     // 4. a QOS configuration
//!     //
//!     // Let's make each of these.
//!
//!     // make the name
//!     let topic_name: Name =
//!         Name::new("/", "some_name").expect("topic name should follow conventions");
//!
//!     // Message types are somewhat complex in ROS 2. In ROS 2, they're typically
//!     // `.msg` files, but this library uses the `Message` trait.
//!     //
//!     // If you need to generate Rust types for your `.msg` files, you can
//!     // use the `msggen` tool in the GitHub repo.
//!     //
//!     // That said, we'll use a message type included with ROS 2 - 'String'!
//!     let message_ty_name = MessageTypeName::new("std_msgs", "String");
//!
//!     // Now, we can make our QOS. These describe how the topic will treat
//!     // the network, and the DDS' expectations around that.
//!     //
//!     // In any case, the default subscription QOS will work great for this
//!     // here since we're making our Node a subscriber.
//!     let qos = &DEFAULT_PUBLISHER_QOS;
//!
//!     // Lastly, we can make the Topic with all that stuff:
//!     node.create_topic(&topic_name, message_ty_name, qos)
//!         .expect("topic creation")
//! };
//!
//! // Now that it's done, we can subscribe our node to the topic!
//! let subscription: Subscription<String> = node
//!     .create_subscription(&topic, None)
//!     .expect("topic subscription should succeed");
//!
//! # return; // runs forever and blocks doctest, so we'll just compile it :)
//!
//! // Each time we get a message, we'll print to the terminal...
//! //
//! // This runs forever!
//! while let Ok((text, _msg_info)) = subscription.async_take().await {
//!     println!("{text}");
//! }
//! # }
//! ```
//!
//! ## DDS
//!
//! The underlying DDS implementation, [`rustdds`](https://atostek.com/en/products/rustdds/), is also made in native Rust.

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
            DomainParticipant, Duration as DdsDuration, QosPolicies, QosPolicyBuilder, Timestamp,
        };
    }
}
