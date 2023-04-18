use rustdds::{*};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{
  message::Message,
  service::{AService, Client, Server,},
  Subscription, Publisher,
};

pub trait ActionTypes {
  type GoalType: Message + Clone; // Used by client to set a goal for the server
  type ResultType: Message + Clone; // Used by server to report result when action ends
  type FeedbackType: Message; // Used by server to report progrss during action excution

  fn goal_type_name() -> String;
  fn result_type_name() -> String;
  fn feedback_type_name() -> String;  
}


pub struct ActionClientQosPolicies {
  goal_service_qos: QosPolicies,
  result_service_qos: QosPolicies,
  cancel_service_qos: QosPolicies,
  feedback_subscription_qos: QosPolicies,
  status_subscription_qos: QosPolicies,
}

pub struct ActionServerQosPolicies {
  goal_service_qos: QosPolicies,
  result_service_qos: QosPolicies,
  cancel_service_qos: QosPolicies,
  feedback_publication_qos: QosPolicies,
  status_publication_qos: QosPolicies,
}

#[derive(Clone, Serialize, Deserialize)]
struct GoalResponse {} // placeholder
impl Message for GoalResponse {}

#[derive(Clone, Serialize, Deserialize)]
struct GoalStatusArray {} // placeholder
impl Message for GoalStatusArray {}


pub struct ActionClient<A> 
where 
  A: ActionTypes,
  A::GoalType: Message + Clone,
  A::ResultType: Message + Clone,
  A::FeedbackType: Message,
{
  goal_client: 
    Client<AService<A::GoalType,GoalResponse>>,
  //cancel_client: ,
  //result_client: ,
  feedback_subscription: Subscription<A::FeedbackType>, 
    // but it is called packagename/action/ActionName_FeedbackMEssage
  status_subscription: Subscription<GoalStatusArray>, 

  action_name: String,
}

impl<A> ActionClient<A> 
where
  A: ActionTypes,
  A::GoalType: Message + Clone,
  A::ResultType: Message + Clone,
  A::FeedbackType: Message,
{

}


// pub struct ActionServer<A> 
// where A: ActionTypes,
// {
//   a: Phant
// }

// impl<A> ActionServer<A> {

// }