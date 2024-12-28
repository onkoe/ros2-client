# Changelog

This documents changes to each version of the library. Newest versions are up top.

## New in Version 0.7

* `NodeName` namespace is no longer allowed to be the empty string, as it confuses ROS 2 tools. Minimum namespace is "/".
* Parameter support, incl. Parameter services
* Time support

### 0.7.1

* Subscribers can `take()` samples with deserialization "seed" value.
This allows more run-time control of deserialization. Upgrade to RustDDS 0.10.0.

### 0.7.2

* Adapt to separation of CDR encoding from RustDDS.

### 0.7.4

* Implement std `Error` trait for `NameError` and `NodeCreateError`
* Async `wait_for_writer` and `wait_for_reader` results now implement `Send`.

### 0.7.5

* New feature `pre-iron-gid`. The `Gid` `.msg` definition has changed between ROS2 Humble and Iron. `ros2-client` now uses the newer version by default. Use this feature to revert to the old definition.

## New in Version 0.6

* Reworked ROS 2 Discovery implementation. Now `Node` has `.status_receiver()`
* Async `.spin()` call to run the Discovery mechanism.
* Now, `Client` has the `.wait_for_service()` method.
* New API for naming Nodes, Topics, Services, Actions, and data types for Topics, Actions, and Services. The new API is more structured to avoid possible confusion and errors from parsing strings.

## New in version 0.5

* Actions are supported
* async programming interface. This should make a built-in event loop unnecessary, as Rust async executors do that already. This means that `ros2-client` is not going to implement a call similar to  [`rclcpp::spin(..)`](https://docs.ros.org/en/rolling/Concepts/Intermediate/About-Executors.html).
