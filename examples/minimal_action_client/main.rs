use std::time::Duration;

use futures::{pin_mut, FutureExt as StdFutureExt, StreamExt};
#[allow(unused_imports)]
use log::{debug, error, info, warn};
use ros2_client::{
    action, action_msgs, ActionTypeName, Context, Name, NodeName, NodeOptions, ServiceMapping,
};
use rustdds::{dds::WriteError, policy, QosPolicies, QosPolicyBuilder};
use smol::future::FutureExt;

// Test / demo program of ROS2 Action, client side.
//
// To set up a server from ROS2:
// % ros2 run examples_rclcpp_minimal_action_server
// action_server_member_functions or
// % ros2 run examples_rclpy_minimal_action_server server_queue_goals
//
// Then run this example.

// This is a ros2-client version of the ROS2 original example programs:
// https://github.com/ros2/examples/blob/rolling/rclcpp/actions/minimal_action_client/member_functions.cpp
// or
// https://github.com/ros2/examples/blob/rolling/rclpy/actions/minimal_action_client/examples_rclpy_minimal_action_client/client_asyncio.py
//
// Unlike the originals, this program loops and sends repeatedly new action
// goals.

// Original action definition
// https://docs.ros2.org/latest/api/action_tutorials_interfaces/action/Fibonacci.html
//
// int32 order
// ---
// int32[] sequence
// ---
// int32[] partial_sequence

// Rust version of action type definition
//
// We define the action using standard/primitive types, but we could
// just as well use e.g.
// struct FibonacciActionGoal{ goal: i32 }
// or any other tuple/struct that contains only an i32.
type FibonacciAction = action::Action<i32, Vec<i32>, Vec<i32>>;

fn main() {
    pretty_env_logger::init();

    // Set Ctrl-C handler
    let (stop_sender, stop_receiver) = smol::channel::bounded(2);
    ctrlc::set_handler(move || {
        // We will send two stop commands, one for reader, the other for writer.
        println!("Stopping.");
        stop_sender.send_blocking(()).unwrap_or(());
    })
    .expect("Error setting Ctrl-C handler");
    println!("Press Ctrl-C to quit.");

    // ROS Context and Node
    let context = Context::new().unwrap();

    let mut node = context
        .new_node(
            NodeName::new("/rustdds", "fibonacci_client").unwrap(),
            NodeOptions::default(),
        )
        .unwrap();

    smol::spawn(node.spinner().unwrap().spin()).detach();

    let service_qos = create_qos();

    let fibonacci_action_qos = action::ActionClientQosPolicies {
        goal_service: service_qos.clone(),
        result_service: service_qos.clone(),
        cancel_service: service_qos.clone(),
        feedback_subscription: service_qos.clone(),
        status_subscription: service_qos,
    };

    let fibonacci_action_client = node
        .create_action_client::<FibonacciAction>(
            ServiceMapping::Enhanced,
            &Name::new("/", "fibonacci").unwrap(),
            &ActionTypeName::new("example_interfaces", "Fibonacci"),
            fibonacci_action_qos,
        )
        .unwrap();

    let mut request_generator = 2; // initialize request generator

    let main_loop = async {
        let mut run = true;
        let mut stop = stop_receiver.recv().fuse();

        let mut tick_stream = // Send new Goal at every tick, if previous one is not running.
      futures::StreamExt::fuse(smol::Timer::interval(Duration::from_secs(1)));

        while run {
            futures::select! {
              _ = stop => run = false,

              _tick = tick_stream.select_next_some() => {
                // Generate a goal for Fibonacci action server
                request_generator += 2;
                request_generator %= 10;
                let order = request_generator;
                println!(">>> Sending goal: {order}");
                // Send a goal for the action server.
                // Wait for the server to accept or reject the goal or timeout
                let goal_response_or_timeout =
                  fibonacci_action_client.async_send_goal(order)
                    .or(async {
                      smol::Timer::after(Duration::from_secs(5)).await;
                      println!(">>> No goal response. Is action server running?");
                      Err(WriteError::WouldBlock { data: () }.into())
                    });
                match goal_response_or_timeout.await
                {
                  Ok((goal_id, goal_response)) => {
                    // Server responded to goal request.
                    println!("<<< Goal Response={:?} goal_id={:?}", goal_response, goal_id);
                    if goal_response.accepted {
                      // Now that we have a goal, we can ask for a result, feedback, and status.
                      let feedback_stream =
                        fibonacci_action_client.feedback_stream(goal_id);
                      pin_mut!(feedback_stream);
                      let status_stream = fibonacci_action_client.all_statuses_stream();
                      pin_mut!(status_stream);
                      let mut goal_finish_timeout =
                        futures::FutureExt::fuse(smol::Timer::interval(Duration::from_secs(30)));
                      let result_fut = fibonacci_action_client.async_request_result(goal_id)
                                          .fuse();
                      pin_mut!(result_fut);

                      let mut goal_done = false;

                      while ! goal_done {
                      futures::select! {
                          _ = stop => { run = false; goal_done=true; },

                          _ = goal_finish_timeout => {
                            goal_done=true;
                            println!("Goal execution timeout. {:?}", goal_id);
                          }

                          // get action result
                          action_result = result_fut => {
                            goal_done = true;
                            match action_result {
                              Ok((goal_status, result)) => {
                                println!("<<< Action Result: {:?} Status: {:?}", result, goal_status);
                              }
                              Err(e) => println!("<<< Action Result error {:?}", e),
                            }
                            println!("\n");
                          }

                          // get action feedback
                          feedback = feedback_stream.select_next_some() => {
                            println!("<<< Feedback: {:?}", feedback);
                          }

                          // get action status changes
                          status = status_stream.select_next_some() => {
                            print!("<<< Status: ");
                            match status {
                              Ok(status) =>
                                match status.status_list.iter().find(|gs| gs.goal_info.goal_id == goal_id) {
                                  Some(action_msgs::GoalStatus{goal_info:_, status}) => println!("{:?}",status),
                                  None => println!("Our status is missing: {:?}", status.status_list),
                                },
                              Err(e) => println!("{:?}",e),
                            }
                          }
                        } // select!
                      } // while goal not done
                    } else {
                      println!("!!! Goal was not accepted. Sulking for a moment.");
                      smol::Timer::after(Duration::from_secs(5)).await;
                    }
                  } // Ok(..)
                  Err(e) => println!("<<< Goal send error {:?}", e),
                } // match
              }
            } // select!
        } // while
        debug!("main loop done");
    };

    // Debugging output:
    //
    // smol::spawn( node.status_receiver().for_each(|event| async move {
    //   println!("{:?}", event);
    // })).detach();

    // run it!
    smol::block_on(main_loop);
}

fn create_qos() -> QosPolicies {
    let service_qos: QosPolicies = {
        QosPolicyBuilder::new()
            .reliability(policy::Reliability::Reliable {
                max_blocking_time: rustdds::Duration::from_millis(100),
            })
            .history(policy::History::KeepLast { depth: 1 })
            .build()
    };
    service_qos
}
