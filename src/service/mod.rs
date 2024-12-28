//! Implementation of ROS 2 [Services](https://docs.ros.org/en/rolling/Tutorials/Beginner-CLI-Tools/Understanding-ROS2-Services/Understanding-ROS2-Services.html)
use std::marker::PhantomData;

use crate::message::Message;

pub mod client;
pub mod parameters;
pub mod request_id;
pub mod server;
pub mod wrappers;

pub use client::Client;
pub use server::Server;

// --------------------------------------------
// --------------------------------------------

/// A Service is a ROS 2 Interface where a requester sends a [`Self::Request`]
/// and a responder makes a computation, returning a [`Self::Response`].
///
/// These are used to represent short-term operations across Nodes in ROS 2.
/// For additional information about ROS 2 services, see
/// [its documentation](https://docs.ros.org/en/rolling/Concepts/Basic/About-Interfaces.html#services).
///
/// Within this crate, this trait pairs the two types together and requires
/// that each is a [`Message`] (can be serialized) with known type names.
///
/// ## Example
///
/// ```
/// use ros2_client::prelude::*;
/// use serde::{Deserialize, Serialize};
///
/// /// A service used to add numbers.
/// struct AdditionService;
///
/// #[derive(Clone, Debug, Deserialize, Serialize)]
/// struct AdditionRequest {
///     pub a: i32,
///     pub b: i32,
/// }
///
/// #[derive(Clone, Debug, Deserialize, Serialize)]
/// struct AdditionResponse {
///     answer: i32,
/// }
///
/// impl Message for AdditionRequest {}
/// impl Message for AdditionResponse {}
///
/// impl Service for AdditionService {
///     type Request = AdditionRequest;
///
///     type Response = AdditionResponse;
///
///     fn request_type_name(&self) -> &str {
///         "AdditionRequest"
///     }
///
///     fn response_type_name(&self) -> &str {
///         "AdditionResponse"
///     }
/// }
/// ```
pub trait Service {
    /// A message sent by the 'requester' containing information about what the
    /// responder should compute.
    type Request: Message;
    /// The reply sent by the 'responder' with info about the computation.
    type Response: Message;
    fn request_type_name(&self) -> &str;
    fn response_type_name(&self) -> &str;
}

// --------------------------------------------
// --------------------------------------------

/// AService is a means of constructing a descriptor for a Service on the fly.
/// This allows generic code to construct a Service from the types of
/// request and response.
pub struct AService<Q, S>
where
    Q: Message,
    S: Message,
{
    q: PhantomData<Q>,
    s: PhantomData<S>,
    req_type_name: String,
    resp_type_name: String,
}

impl<Q, S> AService<Q, S>
where
    Q: Message,
    S: Message,
{
    pub fn new(req_type_name: String, resp_type_name: String) -> Self {
        Self {
            req_type_name,
            resp_type_name,
            q: PhantomData,
            s: PhantomData,
        }
    }
}

impl<Q, S> Service for AService<Q, S>
where
    Q: Message,
    S: Message,
{
    type Request = Q;
    type Response = S;

    fn request_type_name(&self) -> &str {
        &self.req_type_name
    }

    fn response_type_name(&self) -> &str {
        &self.resp_type_name
    }
}

// --------------------------------------------
// --------------------------------------------

/// Selects how Service Requests and Responses are to be mapped to DDS.
///
/// There are different and incompatible ways to map Services onto DDS Topics.
/// In order to interoperate with ROS 2, you have to select the same mapping it
/// uses. The mapping used by ROS2 depends on the DDS implementation used and
/// its configuration.
///
/// For details, see OMG Specification
/// [RPC over DDS](https://www.omg.org/spec/DDS-RPC/1.0/About-DDS-RPC/)
/// Section "7.2.4 Basic and Enhanced Service Mapping for RPC over DDS",
/// which defines Service Mappings "Basic" and "Enhanced".
///
/// ServiceMapping::Cyclone represents a third mapping used by RMW for
/// CycloneDDS.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServiceMapping {
    /// "Basic" service mapping from RPC over DDS specification.
    /// * RTI Connext with `RMW_CONNEXT_REQUEST_REPLY_MAPPING=basic`, but this is
    ///   not tested, so may not work.
    Basic,

    /// "Enhanced" service mapping from RPC over DDS specification.
    /// * ROS2 Foxy with eProsima DDS,
    /// * ROS2 Galactic with RTI Connext (rmw_connextdds, not rmw_connext_cpp) -
    ///   set environment variable `RMW_CONNEXT_REQUEST_REPLY_MAPPING=extended`
    ///   before running ROS2 executable.
    Enhanced,

    /// CycloneDDS-specific service mapping.
    /// Specification for this mapping is unknown, technical details are
    /// reverse-engineered from ROS2 sources.
    /// * ROS2 Galactic with CycloneDDS - Seems to work on the same host only, not
    ///   over actual network.
    Cyclone,
}
