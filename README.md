<!-- cargo-rdme start -->

# `ros2_client`

A ROS 2 client library for Rust. It's similar to [`rclcpp`](https://docs.ros.org/en/rolling/p/rclcpp/) and [`rclpy`](https://docs.ros.org/en/rolling/p/rclpy/), but made in native Rust. It doesn't link to [`rcl`](https://github.com/ros2/rcl) or any other external DDS library. Instead, it uses [`RustDDS`](https://github.com/jhelovuo/RustDDS) for communication.

This library's API builds on the concepts of `rclcpp` and `rclpy`, but it's not identical to avoid some awkward constructs. For example, callbacks in `rclpy` are instead replaced with Rust's `async` interface. However, like those libraries, there is a `spin` call to have `ros2_client` execute background tasks.

## Quick Start

Most of this crate's types are readily available in its `prelude`. You can import it like so:

```rust
use ros2_client::prelude::*;
```

### Example

With the types in that prelude, we can now start building our Nodes and other ROS 2 types! Let's start by writing a simple chat subscriber node:

```rust
use ros2_client::prelude::{dds::*, *};

// In `ros2_client`, the `Context` struct needs to be initialized before you
// can make anything talk.
//
// That's because it holds the DDS layer, which handles all networking here.
let ctx = Context::from_domain_participant(DomainParticipant::new(1).expect("dds security"))
    .expect("there shouldn't be another context");

// Let's make a node!
let mut node = {
    // We'll need a few things first:
    //
    // 1. `ctx` (done)
    // 2. NodeName
    // 3. NodeOptions

    // I'll start with the name:
    let node_name = NodeName::new("/rustdds", "example_node")
        .expect("node name should follow ROS 2 naming conventions");

    // Great! Now, we can focus on the options.
    //
    // Most of the time, we'll want to turn on `rosout`, which allows the node to
    // print into the ROS 2 domain's `rosout` topic.
    //
    // Or, in other words, we can print to a shared 'console'.
    let node_options = NodeOptions::new()
        .enable_rosout(true) // this is on by default but shh...
        .declare_parameter("is_cool", ParameterValue::Boolean(true));

    // Finally, we can make the node:
    ctx.new_node(node_name, node_options)
        .expect("node creation should succeed")
};

// Alright, our node is finished. Let's make a topic for it to subscribe to!
let topic: Topic = {
    // Topics require the following:
    //
    // 1. a Node to create it (done)
    // 2. some Name
    // 3. A known Message type
    // 4. a QOS configuration
    //
    // Let's make each of these.

    // make the name
    let topic_name: Name =
        Name::new("/", "some_name").expect("topic name should follow conventions");

    // Message types are somewhat complex in ROS 2. In ROS 2, they're typically
    // `.msg` files, but this library uses the `Message` trait.
    //
    // If you need to generate Rust types for your `.msg` files, you can
    // use the `msggen` tool in the GitHub repo.
    //
    // That said, we'll use a message type included with ROS 2 - 'String'!
    let message_ty_name = MessageTypeName::new("std_msgs", "String");

    // Now, we can make our QOS. These describe how the topic will treat
    // the network, and the DDS' expectations around that.
    //
    // In any case, the default subscription QOS will work great for this
    // here since we're making our Node a subscriber.
    let qos = &DEFAULT_PUBLISHER_QOS;

    // Lastly, we can make the Topic with all that stuff:
    node.create_topic(&topic_name, message_ty_name, qos)
        .expect("topic creation")
};

// Now that it's done, we can subscribe our node to the topic!
let subscription: Subscription<String> = node
    .create_subscription(&topic, None)
    .expect("topic subscription should succeed");


// Each time we get a message, we'll print to the terminal...
//
// This runs forever!
while let Ok((text, _msg_info)) = subscription.async_take().await {
    println!("{text}");
}
```

**Additional examples are present in the `/examples` folder. Please take a look!**

## Features

- ROS 2
  - ✅ Nodes
  - ✅ Interfaces
    - ✅ Messages
        - 🚧 `.msg` => `.rs` generation experimental
    - ✅ Topics
        - ✅ Publishers
        - ✅ Subscribers
    - ✅ Services (client and server, mostly async)
    - ✅ Actions (async)
  - ✅ Parameters (remote Parameter manipulation)
  - ✅ Serialization (via `serde`)
  - ✅ `rosout` logging
  - ✅ Time (ROS, simulated, steady)
- DDS
  - ✅ Discovery (ROS Graph update events, async)
  - ✅ QoS

## Compatibility (with ROS 2 Releases)

This table shows what is expected to work. Note that older releases are not routinely tested, so a newer release is a better bet.

| ROS 2 Release | `ros2-client` should interoperate? |
| ------------- | :------------ |
| A - E         | Maybe. Not tested. |
| Foxy, Galactic, Humble | Yes. Enable feature `pre-iron-gid` when building `ros2-client` 0.7.5 or newer |
| Iron  | Yes. Not well tested. Requires `ros2-client` 0.7.5 or newer |
| Jazzy | Yes. Requires `ros2-client` 0.7.5 or newer |

## Changelog

Please see [the changelog](./CHANGELOG.md) for info about crate updates.

## Related Work

- [`ros2_rust`](https://github.com/ros2-rust/ros2_rust) is closest(?) to an official ROS2 client library. It links to ROS2 `rcl` library written in C.
- [`rclrust`](https://github.com/rclrust/rclrust) is another ROS2 client library for Rust. It supports ROS2 Services and Topics, and links to ROS2 libraries, e.g. `rcl` and `rmw`.
- [`rus2`](https://github.com/marshalshi/rus2) exists, but appears to be inactive since September 2020.

## License

This crate is licensed under the Apache License, Version 2.0. See the [LICENSE file](./LICENSE) for additional information.

<!-- cargo-rdme end -->
