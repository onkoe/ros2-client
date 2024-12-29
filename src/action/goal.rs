//! Contains types related to goal management (in Actions).
//!
//! For additional information, please see the [`action`] documentation.

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{
    interfaces::{builtin_interfaces::Time, unique_identifier_msgs::UUID},
    message::Message,
};

/// An identifier for an arbitrary 'goal'. Related to [`Action`]s.
pub type GoalId = UUID;

/// From [GoalInfo](https://docs.ros2.org/foxy/api/action_msgs/msg/GoalInfo.html)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GoalInfo {
    /// A unique identifier for a goal.
    ///
    /// These are used to track a goal's progress, even in complex systems.
    pub goal_id: GoalId,
    /// Time when the goal was accepted.
    pub stamp: Time,
}
impl Message for GoalInfo {}

/// The status of a goal's completion within an Action.
#[derive(Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(i8)]
pub enum GoalStatusEnum {
    /// The goal's status is unknown, or the Action Server has not yet
    /// responded to the request.
    ///
    /// Also used for new goals.
    Unknown = 0,

    /// The goal has been accepted by the Action Server and is awaiting
    /// execution.
    Accepted = 1,

    /// The Action Server is currently performing the Action but has not yet
    /// reported completion.
    Executing = 2,

    /// The Client requested that the goal is canceled, and the Action Server
    /// accepted the cancel request.
    Canceling = 3,

    /// The Action was successfully completed!
    Succeeded = 4,

    /// The Action execution was canceled by request of an Action Client.
    Canceled = 5,

    /// The Action was aborted by the Action Server without an external request.
    Aborted = 6,
}

/// From [GoalStatus](https://docs.ros2.org/foxy/api/action_msgs/msg/GoalStatus.html)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GoalStatus {
    /// Information about the goal, with ID and timestamp.
    pub goal_info: GoalInfo,
    /// Status of the internal state machine toward goal completion.
    pub status: GoalStatusEnum,
}
impl Message for GoalStatus {}

/// A collection of goal statuses.
///
/// From [GoalStatusArray](https://docs.ros2.org/foxy/api/action_msgs/msg/GoalStatusArray.html)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GoalStatusArray {
    /// The list of statuses.
    pub status_list: Vec<GoalStatus>,
}
impl Message for GoalStatusArray {}

/// From [CancelGoal](https://docs.ros2.org/foxy/api/action_msgs/srv/CancelGoal.html)
// Cancel one or more goals with the following policy:
//
// - If the goal ID is zero and timestamp is zero, cancel all goals.
// - If the goal ID is zero and timestamp is not zero, cancel all goals accepted at or before the
//   timestamp.
// - If the goal ID is not zero and timestamp is zero, cancel the goal with the given ID regardless
//   of the time it was accepted.
// - If the goal ID is not zero and timestamp is not zero, cancel the goal with the given ID and all
//   goals accepted at or before the timestamp.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CancelGoalRequest {
    pub(crate) goal_info: GoalInfo,
}
impl Message for CancelGoalRequest {}

/// From [CancelGoal](https://docs.ros2.org/foxy/api/action_msgs/srv/CancelGoal.html)
#[derive(Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq, Debug)]
#[repr(i8)]
pub enum CancelGoalResponseEnum {
    // Doc comments here copied from ROS2 message definition.
    /// Indicates the request was accepted without any errors.
    /// One or more goals have transitioned to the CANCELING state.
    /// The goals_canceling list is not empty.
    None = 0,

    /// Indicates the request was rejected.
    /// No goals have transitioned to the CANCELING state. The goals_canceling
    /// list is empty.
    Rejected = 1,

    /// Indicates the requested goal ID does not exist.
    /// No goals have transitioned to the CANCELING state. The goals_canceling
    /// list is empty.
    UnknownGoal = 2,

    /// Indicates the goal is not cancelable because it is already in a terminal
    /// state. No goals have transitioned to the CANCELING state. The
    /// goals_canceling list is empty.
    GoalTerminated = 3,
}

/// A response indicating the Action Server's reply to a [`CancelGoalRequest`].
///
/// From [CancelGoal](https://docs.ros2.org/foxy/api/action_msgs/srv/CancelGoal.html)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CancelGoalResponse {
    /// A return code indicating what the Action Server decided on.
    pub return_code: CancelGoalResponseEnum,
    /// What goals are canceling according to the request, if any.
    pub goals_canceling: Vec<GoalInfo>,
}
impl Message for CancelGoalResponse {}
