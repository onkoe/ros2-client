use futures::StreamExt;

use log::debug;
use ros2_client::prelude::{dds::*, *};
use serde::{Deserialize, Serialize};

// This is an example / test program.
// Test this against minimal_client found in
// https://github.com/ros2/examples/blob/master/rclpy/services/minimal_client/examples_rclpy_minimal_client/client.py
// or
// % ros2 run examples_rclpy_minimal_client client
// or
// % ros2 run examples_rclcpp_minimal_client client_main

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTwoIntsRequest {
    pub a: i64,
    pub b: i64,
}
impl Message for AddTwoIntsRequest {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTwoIntsResponse {
    pub sum: i64,
}
impl Message for AddTwoIntsResponse {}

fn main() {
    pretty_env_logger::init();

    debug!(">>> ros2_service starting...");
    let mut node = create_node();
    let service_qos = create_qos();

    println!(">>> ros2_service node started");

    let server = node
        .create_server::<AService<AddTwoIntsRequest, AddTwoIntsResponse>>(
            ServiceMapping::Enhanced,
            &Name::new("/", "add_two_ints").unwrap(),
            &ServiceTypeName::new("example_interfaces", "AddTwoInts"),
            service_qos.clone(),
            service_qos,
        )
        .unwrap();

    println!(">>> ros2_service server created");

    let server_stream = server.receive_request_stream().then(|result| async {
        match result {
            Ok((req_id, req)) => {
                println!("request: {} + {}", req.a, req.b);
                let resp = AddTwoIntsResponse { sum: req.a + req.b };
                let sr = server.async_send_response(req_id, resp).await;
                if let Err(e) = sr {
                    println!("Send error {:?}", e);
                }
            }
            Err(e) => println!("Receive request error: {:?}", e),
        }
    });

    // run it!
    smol::block_on(server_stream.count());
    // .count() here just converts Stream to ordinary Future.
    // It would return the count of requestes processed, if the stream would end.
} // main

fn create_qos() -> QosPolicies {
    let service_qos: QosPolicies = {
        QosPolicyBuilder::new()
            .history(History::KeepLast { depth: 10 })
            .reliability(Reliability::Reliable {
                max_blocking_time: DdsDuration::from_millis(100),
            })
            .durability(Durability::Volatile)
            .deadline(Deadline(DdsDuration::INFINITE))
            .lifespan(Lifespan {
                duration: DdsDuration::INFINITE,
            })
            .liveliness(Liveliness::Automatic {
                lease_duration: DdsDuration::INFINITE,
            })
            .build()
    };
    service_qos
}

fn create_node() -> Node {
    let context = Context::new().unwrap();
    context
        .new_node(
            NodeName::new("/rustdds", "rustdds_server").unwrap(),
            NodeOptions::new().enable_rosout(true),
        )
        .unwrap()
}
