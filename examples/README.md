# Examples

## Example: minimal_action_server and minimal_action_client

These are re-implementations of [similarly named ROS examples](https://docs.ros.org/en/iron/Tutorials/Intermediate/Writing-an-Action-Server-Client/Cpp.html). They should be interoperable with ROS 2 example programs in C++ or Python.

To test this, start a server and then, in a separate terminal, a client, e.g.

`ros2 run examples_rclcpp_minimal_action_server action_server_member_functions`
and
`cargo run --example=minimal_action_client`

or

`cargo run --example=minimal_action_server`
and
`ros2 run examples_rclpy_minimal_action_client client`

You should see the client requesting for a sequence of Fibonacci numbers, and the server providing them until the requested sequence length is reached.

## Example: turtle_teleop

The included example program should be able to communicate with out-of-the-box ROS2 turtlesim example.

Install ROS2 and start the simulator by `ros2 run turtlesim turtlesim_node`. Then run the `turtle_teleop` example to control the simulator.

![Turtlesim screenshot](examples/turtle_teleop/screenshot.png)

Teleop example program currently has the following keyboard commands:

* Cursor keys: Move turtle
* `q` or `Ctrl-C`: quit
* `r`: reset simulator
* `p`: change pen color (for turtle1 only)
* `a`/`b` : spawn turtle1 / turtle2
* `A`/`B` : kill turtle1 / turtle2
* `1`/`2` : switch control between turtle1 / turtle2
* `d`/`f`/`g`: Trigger or cancel absolute rotation action.

## Example: ros2_service_server

Install ROS2. This has been tested to work against "Galactic" release, using either eProsima FastDDS or RTI Connext DDS (`rmw_connextdds`, not `rmw_connext_cpp`).

Start server: `cargo run --example=ros2_service_server`

In another terminal or computer, run a client: `ros2 run examples_rclpy_minimal_client client`

## Example: ros2_service_client

Similar to above.

Start server: `ros2 run examples_rclpy_minimal_service service`

Run client: `cargo run --example=ros2_service_client`
