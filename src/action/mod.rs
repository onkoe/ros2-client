//! Types and utilities for ROS 2 Actions.

use std::{
    collections::{btree_map::Entry, BTreeMap},
    marker::PhantomData,
};

use futures::{
    pin_mut,
    stream::{FusedStream, StreamExt},
    Future,
};
use rustdds::{
    dds::{ReadError, ReadResult, WriteError, WriteResult},
    QosPolicies,
};
use serde::{Deserialize, Serialize};

use crate::{
    action::goal::{CancelGoalRequest, CancelGoalResponse, GoalId, GoalInfo, GoalStatusEnum},
    interfaces::{
        builtin_interfaces::{self, Time},
        unique_identifier_msgs::UUID,
    },
    message::Message,
    prelude::{Name, Publisher, Subscription},
    service::{
        client::{CallServiceError, Client},
        request_id::RmwRequestId,
        server::Server,
        AService,
    },
};

pub mod goal;

/// A trait to define an Action type
pub trait ActionTypes {
    /// Used by an Action Client to set a goal for the Action Server.
    type GoalType: Message + Clone;
    /// Used by the Server to report a result when the Action completes.
    type ResultType: Message + Clone;
    /// Used by the Server to report progress during Action execution.
    type FeedbackType: Message;

    /// The type name of the goal message.
    fn goal_type_name(&self) -> &str;
    /// Result message type name.
    fn result_type_name(&self) -> &str;
    /// Feedback message type name.
    fn feedback_type_name(&self) -> &str;
}

/// This is used to construct an ActionType implementation from pre-existing
/// component types.
pub struct Action<G, R, F> {
    g: PhantomData<G>,
    r: PhantomData<R>,
    f: PhantomData<F>,
    goal_typename: String,
    result_typename: String,
    feedback_typename: String,
}

impl<Goal, Result, Feedback> Action<Goal, Result, Feedback>
where
    Goal: Message + Clone,
    Result: Message + Clone,
    Feedback: Message,
{
    /// Given these type names (mentioned in [`ActionTypes`]), this constructor
    /// creates a new [`ActionType`] implementation.
    pub fn new(goal_typename: String, result_typename: String, feedback_typename: String) -> Self {
        Self {
            goal_typename,
            result_typename,
            feedback_typename,
            g: PhantomData,
            r: PhantomData,
            f: PhantomData,
        }
    }
}

impl<G, R, F> ActionTypes for Action<G, R, F>
where
    G: Message + Clone,
    R: Message + Clone,
    F: Message,
{
    type GoalType = G;
    type ResultType = R;
    type FeedbackType = F;

    fn goal_type_name(&self) -> &str {
        &self.goal_typename
    }

    fn result_type_name(&self) -> &str {
        &self.result_typename
    }

    fn feedback_type_name(&self) -> &str {
        &self.feedback_typename
    }
}

//TODO: Make fields private, add constructor and accessors.

/// Collection of QoS policies required for an Action client.
#[derive(Debug, Clone)]
#[expect(missing_docs)]
pub struct ActionClientQosPolicies {
    pub goal_service: QosPolicies,
    pub result_service: QosPolicies,
    pub cancel_service: QosPolicies,
    pub feedback_subscription: QosPolicies,
    pub status_subscription: QosPolicies,
}

/// Collection of QoS policies requires for an Action server
#[derive(Debug, Clone)]
#[expect(missing_docs)]
pub struct ActionServerQosPolicies {
    pub goal_service: QosPolicies,
    pub result_service: QosPolicies,
    pub cancel_service: QosPolicies,
    pub feedback_publisher: QosPolicies,
    pub status_publisher: QosPolicies,
}

/// A request message for the goal sending service.
///
/// (emulating ROS2 IDL code generator: Goal sending/setting service)
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SendGoalRequest<Goal> {
    /// A goal's unique ID.
    pub goal_id: GoalId,
    /// A goal.
    pub goal: Goal,
}
impl<G: Message> Message for SendGoalRequest<G> {}

/// A response message for the goal sending service.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SendGoalResponse {
    /// If the goal was accepted, this is `true`.
    pub accepted: bool,
    /// Timestamp.
    pub stamp: Time,
}
impl Message for SendGoalResponse {}

/// A request message for the result getting service.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetResultRequest {
    /// The goal's unique ID.
    pub goal_id: GoalId,
}
impl Message for GetResultRequest {}

/// A response message for the result getting service.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct GetResultResponse<Res> {
    /// Information about the goal's status.
    pub status: GoalStatusEnum, // interpretation same as in GoalStatus message?
    /// The result message.
    pub result: Res,
}
impl<R: Message> Message for GetResultResponse<R> {}

/// A message type for the feedback topic.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FeedbackMessage<Feedback> {
    /// The goal's unique ID.
    pub goal_id: GoalId,
    /// The actual feedback message.
    pub feedback: Feedback,
}
impl<F: Message> Message for FeedbackMessage<F> {}

/// An Action Client.
pub struct ActionClient<A>
where
    A: ActionTypes,
    A::GoalType: Message + Clone,
    A::ResultType: Message + Clone,
    A::FeedbackType: Message,
{
    pub(crate) my_goal_client: Client<AService<SendGoalRequest<A::GoalType>, SendGoalResponse>>,

    pub(crate) my_cancel_client:
        Client<AService<goal::CancelGoalRequest, goal::CancelGoalResponse>>,

    pub(crate) my_result_client:
        Client<AService<GetResultRequest, GetResultResponse<A::ResultType>>>,

    pub(crate) my_feedback_subscription: Subscription<FeedbackMessage<A::FeedbackType>>,

    pub(crate) my_status_subscription: Subscription<goal::GoalStatusArray>,

    pub(crate) my_action_name: Name,
}

impl<A> ActionClient<A>
where
    A: ActionTypes,
    A::GoalType: Message + Clone,
    A::ResultType: Message + Clone,
    A::FeedbackType: Message,
{
    /// Returns the Action name.
    pub fn name(&self) -> &Name {
        &self.my_action_name
    }

    /// Returns a mutable reference to the goal Client.
    pub fn goal_client(
        &mut self,
    ) -> &mut Client<AService<SendGoalRequest<A::GoalType>, SendGoalResponse>> {
        &mut self.my_goal_client
    }

    /// Returns a mutable reference to the cancel Client.
    pub fn cancel_client(
        &mut self,
    ) -> &mut Client<AService<goal::CancelGoalRequest, goal::CancelGoalResponse>> {
        &mut self.my_cancel_client
    }

    /// Returns a mutable reference to the result Client.
    pub fn result_client(
        &mut self,
    ) -> &mut Client<AService<GetResultRequest, GetResultResponse<A::ResultType>>> {
        &mut self.my_result_client
    }

    /// Returns a mutable reference to the feedback Subscription.
    pub fn feedback_subscription(&mut self) -> &mut Subscription<FeedbackMessage<A::FeedbackType>> {
        &mut self.my_feedback_subscription
    }

    /// Returns a mutable reference to the status Subscription.
    pub fn status_subscription(&mut self) -> &mut Subscription<goal::GoalStatusArray> {
        &mut self.my_status_subscription
    }

    /// Returns the IDs for both the Request and the Goal.
    ///
    /// The Request ID can be used to recognize the correct response from the
    /// Action Server.
    ///
    /// The Goal ID is later used to communicate Goal status and result.
    pub fn send_goal(&self, goal: A::GoalType) -> WriteResult<(RmwRequestId, GoalId), ()>
    where
        <A as ActionTypes>::GoalType: 'static,
    {
        let goal_id = UUID::new_random();
        self.my_goal_client
            .send_request(SendGoalRequest { goal_id, goal })
            .map(|req_id| (req_id, goal_id))
    }

    /// Attempts to receive a response for the specified goal request.
    ///
    /// This will be `None` if the response is not yet available.
    pub fn receive_goal_response(
        &self,
        req_id: RmwRequestId,
    ) -> ReadResult<Option<SendGoalResponse>>
    where
        <A as ActionTypes>::GoalType: 'static,
    {
        loop {
            match self.my_goal_client.receive_response() {
                Err(e) => break Err(e),
                Ok(None) => break Ok(None), // not yet
                Ok(Some((incoming_req_id, resp))) if incoming_req_id == req_id =>
                // received the expected answer
                {
                    break Ok(Some(resp))
                }
                Ok(Some((incoming_req_id, _resp))) => {
                    // got someone else's answer. Try again.
                    log::info!(
                        "Goal Response not for us: {:?} != {:?}",
                        incoming_req_id,
                        req_id
                    );
                    continue;
                }
            }
        }
        // We loop here to drain all the answers received so far.
        // The mio .poll() only does not trigger again for the next item, if it has
        // been received already.
    }

    /// Sends a goal to the Action Server.
    pub async fn async_send_goal(
        &self,
        goal: A::GoalType,
    ) -> Result<(GoalId, SendGoalResponse), CallServiceError<()>>
    where
        <A as ActionTypes>::GoalType: 'static,
    {
        let goal_id = UUID::new_random();
        let send_goal_response =
            Client::async_call_service(&self.my_goal_client, SendGoalRequest { goal_id, goal })
                .await?;
        Ok((goal_id, send_goal_response))
    }

    /// From ROS2 docs:
    /// https://docs.ros2.org/foxy/api/action_msgs/srv/CancelGoal.html
    ///
    /// Cancel one or more goals with the following policy:
    /// - If the goal ID is zero and timestamp is zero, cancel all goals.
    /// - If the goal ID is zero and timestamp is not zero, cancel all goals accepted
    ///   at or before the timestamp.
    /// - If the goal ID is not zero and timestamp is zero, cancel the goal with the
    ///   given ID regardless of the time it was accepted.
    /// - If the goal ID is not zero and timestamp is not zero, cancel the goal with
    ///   the given ID and all goals accepted at or before the timestamp.
    fn cancel_goal_raw(&self, goal_id: GoalId, timestamp: Time) -> WriteResult<RmwRequestId, ()> {
        let goal_info = GoalInfo {
            goal_id,
            stamp: timestamp,
        };
        self.my_cancel_client
            .send_request(CancelGoalRequest { goal_info })
    }

    /// Cancels a goal with the given ID.
    pub fn cancel_goal(&self, goal_id: GoalId) -> WriteResult<RmwRequestId, ()> {
        self.cancel_goal_raw(goal_id, Time::ZERO)
    }

    /// Cancels all goals accepted before the given timestamp.
    pub fn cancel_all_goals_before(&self, timestamp: Time) -> WriteResult<RmwRequestId, ()> {
        self.cancel_goal_raw(GoalId::ZERO, timestamp)
    }

    /// Cancels all goals.
    pub fn cancel_all_goals(&self) -> WriteResult<RmwRequestId, ()> {
        self.cancel_goal_raw(GoalId::ZERO, Time::ZERO)
    }

    /// Attempts to receive a response for the specified cancel request.
    pub fn receive_cancel_response(
        &self,
        cancel_request_id: RmwRequestId,
    ) -> ReadResult<Option<CancelGoalResponse>> {
        loop {
            match self.my_cancel_client.receive_response()? {
                // no reponse yet!
                None => break Ok(None),

                // we got the expected answer!
                Some((incoming_req_id, resp)) if incoming_req_id == cancel_request_id => {
                    break Ok(Some(resp))
                }

                // got someone else's answer. try again.
                Some(_) => continue,
            }
        }
    }

    /// Cancels a goal with the given ID and timestamp.
    pub fn async_cancel_goal(
        &self,
        goal_id: GoalId,
        timestamp: Time,
    ) -> impl Future<Output = Result<CancelGoalResponse, CallServiceError<()>>> + '_ {
        let goal_info = GoalInfo {
            goal_id,
            stamp: timestamp,
        };
        self.my_cancel_client
            .async_call_service(CancelGoalRequest { goal_info })
    }

    /// Requests the Result for the goal with the given ID.
    pub fn request_result(&self, goal_id: GoalId) -> WriteResult<RmwRequestId, ()>
    where
        <A as ActionTypes>::ResultType: 'static,
    {
        self.my_result_client
            .send_request(GetResultRequest { goal_id })
    }

    /// Attempts to receive the result for the specified request.
    pub fn receive_result(
        &self,
        result_request_id: RmwRequestId,
    ) -> ReadResult<Option<(GoalStatusEnum, A::ResultType)>>
    where
        <A as ActionTypes>::ResultType: 'static,
    {
        loop {
            match self.my_result_client.receive_response()? {
                // not yet
                None => break Ok(None),

                // we got the expected answer!
                Some((incoming_req_id, GetResultResponse { status, result }))
                    if incoming_req_id == result_request_id =>
                {
                    break Ok(Some((status, result)))
                }

                // got someone else's answer. try again.
                Some(_) => continue,
            }
        }
    }

    /// Asynchronously request goal Result.
    ///
    /// Result should be requested as soon as a goal is accepted, but will only
    /// be received when the Server informs that the goal has either Succeeded,
    /// or has been Canceled/Aborted.
    pub async fn async_request_result(
        &self,
        goal_id: GoalId,
    ) -> Result<(GoalStatusEnum, A::ResultType), CallServiceError<()>>
    where
        <A as ActionTypes>::ResultType: 'static,
    {
        let GetResultResponse { status, result } = self
            .my_result_client
            .async_call_service(GetResultRequest { goal_id })
            .await?;
        Ok((status, result))
    }

    /// Attempts to receive a Feedback message for the goal with the given ID.
    pub fn receive_feedback(&self, goal_id: GoalId) -> ReadResult<Option<A::FeedbackType>>
    where
        <A as ActionTypes>::FeedbackType: 'static,
    {
        loop {
            match self.my_feedback_subscription.take()? {
                None => break Ok(None),

                Some((fb_msg, _msg_info)) if fb_msg.goal_id == goal_id => {
                    break Ok(Some(fb_msg.feedback))
                }

                Some((fb_msg, _msg_info)) => {
                    // feedback on some other goal
                    log::debug!(
                        "Feedback on another goal {:?} != {goal_id:?}",
                        fb_msg.goal_id
                    )
                }
            }
        }
    }

    /// Receive asynchronous feedback stream of goal progress.
    pub fn feedback_stream(
        &self,
        goal_id: GoalId,
    ) -> impl FusedStream<Item = ReadResult<A::FeedbackType>> + '_
    where
        <A as ActionTypes>::FeedbackType: 'static,
    {
        let expected_goal_id = goal_id; // rename
        self.my_feedback_subscription
            .async_stream()
            .filter_map(move |result| async move {
                match result {
                    Err(e) => Some(Err(e)),
                    Ok((FeedbackMessage { goal_id, feedback }, _msg_info)) => {
                        if goal_id == expected_goal_id {
                            Some(Ok(feedback))
                        } else {
                            log::debug!("Feedback for some other {:?}.", goal_id);
                            None
                        }
                    }
                }
            })
    }

    /// Attempts to receive the status of all Goals.
    ///
    /// Note that this doesn't take a Goal ID. Thus, it reports all Goal
    /// statuses.
    //
    // FIXME: the `Option` in ret is unclear and should be `Result`.
    #[tracing::instrument(skip(self))]
    pub fn receive_status(&self) -> ReadResult<Option<goal::GoalStatusArray>> {
        self.my_status_subscription
            .take()
            .inspect_err(|e| {
                tracing::error!("Action status subscription failed to deser. message. (see: {e})");
            })
            .map(|res| res.map(|(status_array, _)| status_array))
    }

    /// Attempts to receive the status of all Goals, asynchronously.
    pub async fn async_receive_status(&self) -> ReadResult<goal::GoalStatusArray> {
        let (status_array, _) =
            self.my_status_subscription
                .async_take()
                .await
                .inspect_err(|e| {
                    tracing::error!(
                        "Action status subscription failed to deser. message. (see: {e})"
                    );
                })?;

        Ok(status_array)
    }

    /// Async Stream of status updates
    /// Action server send updates containing status of all goals, hence an array.
    pub fn all_statuses_stream(
        &self,
    ) -> impl FusedStream<Item = ReadResult<goal::GoalStatusArray>> + '_ {
        self.my_status_subscription
            .async_stream()
            .map(|result| result.map(|(gsa, _mi)| gsa))
    }

    /// Returns the status stream for the specfied goal.
    ///
    /// Stream types come from the [`futures`] crate.
    pub fn status_stream(
        &self,
        goal_id: GoalId,
    ) -> impl FusedStream<Item = ReadResult<goal::GoalStatus>> + '_ {
        self.all_statuses_stream()
            .filter_map(move |result| async move {
                match result {
                    Err(e) => Some(Err(e)),
                    Ok(gsa) => gsa
                        .status_list
                        .into_iter()
                        .find(|gs| gs.goal_info.goal_id == goal_id)
                        .map(Ok),
                }
            })
    }
} // impl

// Example topic names and types at DDS level:

// rq/turtle1/rotate_absolute/_action/send_goalRequest :
// turtlesim::action::dds_::RotateAbsolute_SendGoal_Request_ rr/turtle1/
// rotate_absolute/_action/send_goalReply :
// turtlesim::action::dds_::RotateAbsolute_SendGoal_Response_

// rq/turtle1/rotate_absolute/_action/cancel_goalRequest  :
// action_msgs::srv::dds_::CancelGoal_Request_ rr/turtle1/rotate_absolute/
// _action/cancel_goalReply  : action_msgs::srv::dds_::CancelGoal_Response_

// rq/turtle1/rotate_absolute/_action/get_resultRequest :
// turtlesim::action::dds_::RotateAbsolute_GetResult_Request_ rr/turtle1/
// rotate_absolute/_action/get_resultReply :
// turtlesim::action::dds_::RotateAbsolute_GetResult_Response_

// rt/turtle1/rotate_absolute/_action/feedback :
// turtlesim::action::dds_::RotateAbsolute_FeedbackMessage_

// rt/turtle1/rotate_absolute/_action/status :
// action_msgs::msg::dds_::GoalStatusArray_

/// An Action Server.
pub struct ActionServer<A>
where
    A: ActionTypes,
    A::GoalType: Message + Clone,
    A::ResultType: Message + Clone,
    A::FeedbackType: Message,
{
    pub(crate) my_goal_server: Server<AService<SendGoalRequest<A::GoalType>, SendGoalResponse>>,

    pub(crate) my_cancel_server:
        Server<AService<goal::CancelGoalRequest, goal::CancelGoalResponse>>,

    pub(crate) my_result_server:
        Server<AService<GetResultRequest, GetResultResponse<A::ResultType>>>,

    pub(crate) my_feedback_publisher: Publisher<FeedbackMessage<A::FeedbackType>>,

    pub(crate) my_status_publisher: Publisher<goal::GoalStatusArray>,

    pub(crate) my_action_name: Name,
}

impl<A> ActionServer<A>
where
    A: ActionTypes,
    A::GoalType: Message + Clone,
    A::ResultType: Message + Clone,
    A::FeedbackType: Message,
{
    /// Returns the Action name.
    pub fn name(&self) -> &Name {
        &self.my_action_name
    }

    /// Returns a mutable reference to the goal Server.
    pub fn goal_server(
        &mut self,
    ) -> &mut Server<AService<SendGoalRequest<A::GoalType>, SendGoalResponse>> {
        &mut self.my_goal_server
    }

    /// Returns a mutable reference to the cancel Server.
    pub fn cancel_server(
        &mut self,
    ) -> &mut Server<AService<goal::CancelGoalRequest, goal::CancelGoalResponse>> {
        &mut self.my_cancel_server
    }

    /// Returns a mutable reference to the result Server.
    pub fn result_server(
        &mut self,
    ) -> &mut Server<AService<GetResultRequest, GetResultResponse<A::ResultType>>> {
        &mut self.my_result_server
    }

    /// Returns a mutable reference to the Feedback Publisher.
    pub fn feedback_publisher(&mut self) -> &mut Publisher<FeedbackMessage<A::FeedbackType>> {
        &mut self.my_feedback_publisher
    }

    /// Returns a mutable reference to the Status Publisher.
    pub fn my_status_publisher(&mut self) -> &mut Publisher<goal::GoalStatusArray> {
        &mut self.my_status_publisher
    }

    /// Receive a new goal, if available.
    pub fn receive_goal(&self) -> ReadResult<Option<(RmwRequestId, SendGoalRequest<A::GoalType>)>>
    where
        <A as ActionTypes>::GoalType: 'static,
    {
        self.my_goal_server.receive_request()
    }

    /// Send a response for the specified goal request
    pub fn send_goal_response(
        &self,
        req_id: RmwRequestId,
        resp: SendGoalResponse,
    ) -> WriteResult<(), ()>
    where
        <A as ActionTypes>::GoalType: 'static,
    {
        self.my_goal_server.send_response(req_id, resp)
    }

    /// Receive a cancel request, if available.
    pub fn receive_cancel_request(
        &self,
    ) -> ReadResult<Option<(RmwRequestId, goal::CancelGoalRequest)>> {
        self.my_cancel_server.receive_request()
    }

    /// Responds to a received cancel request by sending a cancel response.
    pub fn send_cancel_response(
        &self,
        req_id: RmwRequestId,
        resp: goal::CancelGoalResponse,
    ) -> WriteResult<(), ()> {
        self.my_cancel_server.send_response(req_id, resp)
    }

    /// Receive a result request, if available.
    pub fn receive_result_request(&self) -> ReadResult<Option<(RmwRequestId, GetResultRequest)>>
    where
        <A as ActionTypes>::ResultType: 'static,
    {
        self.my_result_server.receive_request()
    }

    /// Send a result message to the Client.
    pub fn send_result(
        &self,
        result_request_id: RmwRequestId,
        resp: GetResultResponse<A::ResultType>,
    ) -> WriteResult<(), ()>
    where
        <A as ActionTypes>::ResultType: 'static,
    {
        self.my_result_server.send_response(result_request_id, resp)
    }

    /// Send a feedback message to the Client.
    pub fn send_feedback(
        &self,
        goal_id: GoalId,
        feedback: A::FeedbackType,
    ) -> WriteResult<(), FeedbackMessage<A::FeedbackType>> {
        self.my_feedback_publisher
            .publish(FeedbackMessage { goal_id, feedback })
    }

    /// Sends the status of all known goals.
    pub fn send_goal_statuses(
        &self,
        goal_statuses: goal::GoalStatusArray,
    ) -> WriteResult<(), goal::GoalStatusArray> {
        self.my_status_publisher.publish(goal_statuses)
    }
} // impl

/// One of many handles to a goal.  
pub trait GoalHandle {
    /// Returns the goal ID.
    fn goal_id(&self) -> GoalId;
}

/// A handle to a new goal.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct NewGoalHandle<Goal> {
    inner: InnerGoalHandle<Goal>,
    req_id: RmwRequestId,
}

impl<Goal> GoalHandle for NewGoalHandle<Goal> {
    fn goal_id(&self) -> GoalId {
        self.inner.goal_id
    }
}

/// A handle to an accepted goal.
#[derive(Clone, Copy)]
pub struct AcceptedGoalHandle<Goal> {
    inner: InnerGoalHandle<Goal>,
}

impl<Goal> GoalHandle for AcceptedGoalHandle<Goal> {
    fn goal_id(&self) -> GoalId {
        self.inner.goal_id
    }
}

/// A handle to an executing goal.
#[derive(Clone, Copy)]
pub struct ExecutingGoalHandle<Goal> {
    inner: InnerGoalHandle<Goal>,
}

impl<Goal> GoalHandle for ExecutingGoalHandle<Goal> {
    fn goal_id(&self) -> GoalId {
        self.inner.goal_id
    }
}

/// An internal handle to a goal.
///
/// This only contains a goal ID alongside a representation of the goal type.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
struct InnerGoalHandle<Goal> {
    goal_id: GoalId,
    phantom: PhantomData<Goal>,
}

/// A handle to a cancel request.
pub struct CancelHandle {
    /// The request ID.
    req_id: RmwRequestId,
    /// The goals to cancel.
    goals: Vec<GoalId>,
}

impl CancelHandle {
    /// An iterator representing the goals to cancel.
    pub fn goals(&self) -> impl Iterator<Item = GoalId> + '_ {
        self.goals.iter().cloned()
    }

    /// Whether this cancel request will cancel the goal with the given ID.
    pub fn contains_goal(&self, goal_id: &GoalId) -> bool {
        self.goals.contains(goal_id)
    }
}

/// The result of a goal ending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalEndStatus {
    /// The goal succeeded.
    Succeeded,
    /// The server aborted the goal before it could complete.
    Aborted,
    /// The goal was canceled gracefully before completion.
    Canceled,
}

/// An error encountered while interacting with an Action.
pub enum GoalError<T> {
    /// The goal ID specified does not exist.
    NoSuchGoal,
    /// The goal was in an unexpected state.
    WrongGoalState,
    /// DDS had an error during a read operation.
    DDSReadError(ReadError),
    /// DDS had an error during a write operation.
    DDSWriteError(WriteError<T>),
}

impl<T> core::fmt::Display for GoalError<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NoSuchGoal => write!(f, "No such goal"),
            Self::WrongGoalState => write!(f, "Wrong goal state"),
            Self::DDSReadError(e) => write!(f, "DDS read error: {e}"),
            Self::DDSWriteError(e) => write!(f, "DDS write error: {e}"),
        }
    }
}

impl<T> core::fmt::Debug for GoalError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        core::fmt::Display::fmt(self, f) // we'll use this one as the 'canonical' fmter
    }
}

impl<T> core::error::Error for GoalError<T> {}

impl<T> From<ReadError> for GoalError<T> {
    fn from(e: ReadError) -> Self {
        GoalError::DDSReadError(e)
    }
}
impl<T> From<WriteError<T>> for GoalError<T> {
    fn from(e: WriteError<T>) -> Self {
        GoalError::DDSWriteError(e)
    }
}

#[derive(Clone, Debug)]
struct AsyncGoal<A>
where
    A: ActionTypes,
{
    status: GoalStatusEnum,
    accepted_time: Option<builtin_interfaces::Time>,
    goal: A::GoalType,
}

/// An asynchronous Action Server.
pub struct AsyncActionServer<A>
where
    A: ActionTypes,
    A::GoalType: Message + Clone,
    A::ResultType: Message + Clone,
    A::FeedbackType: Message,
{
    actionserver: ActionServer<A>,
    goals: BTreeMap<GoalId, AsyncGoal<A>>,
    result_requests: BTreeMap<GoalId, RmwRequestId>,
}

impl<A> AsyncActionServer<A>
where
    A: ActionTypes,
    A::GoalType: Message + Clone,
    A::ResultType: Message + Clone,
    A::FeedbackType: Message,
{
    /// Creates a new [`Self`] from a sync [`ActionServer`].
    pub fn new(actionserver: ActionServer<A>) -> Self {
        AsyncActionServer::<A> {
            actionserver,
            goals: BTreeMap::new(),
            result_requests: BTreeMap::new(),
        }
    }

    /// Returns the goal, if it exists.
    pub fn get_new_goal(&self, handle: NewGoalHandle<A::GoalType>) -> Option<&A::GoalType> {
        self.goals.get(&handle.inner.goal_id).map(|ag| &ag.goal)
    }

    /// Receive a new goal from an action client.
    /// Server should immediately either accept or reject the goal.
    pub async fn receive_new_goal(&mut self) -> ReadResult<NewGoalHandle<A::GoalType>>
    where
        <A as ActionTypes>::GoalType: 'static,
    {
        let (req_id, goal_id) = loop {
            let (req_id, goal_request) = self
                .actionserver
                .my_goal_server
                .async_receive_request()
                .await?;
            match self.goals.entry(goal_request.goal_id) {
                e @ Entry::Vacant(_) => {
                    e.or_insert(AsyncGoal {
                        status: GoalStatusEnum::Unknown,
                        goal: goal_request.goal,
                        accepted_time: None,
                    });
                    break (req_id, goal_request.goal_id);
                }
                Entry::Occupied(_) => {
                    log::error!(
                        "Received duplicate goal_id {:?} , req_id={:?}",
                        goal_request.goal_id,
                        req_id
                    );
                    continue; // just discard this request
                }
            }
        };
        let inner = InnerGoalHandle {
            goal_id,
            phantom: PhantomData,
        };
        Ok(NewGoalHandle { inner, req_id })
    }

    /// Convert a newly received goal into a accepted goal, i.e. accept it
    /// for execution later. Client will be notified of acceptance.
    /// Note: Once the goal is accepted, the server must eventually call
    /// `.send_result_response()` even if the goal is canceled or aborted.
    pub async fn accept_goal(
        &mut self,
        handle: NewGoalHandle<A::GoalType>,
    ) -> Result<AcceptedGoalHandle<A::GoalType>, GoalError<()>>
    where
        A::GoalType: 'static,
    {
        match self.goals.entry(handle.inner.goal_id) {
            Entry::Vacant(_) => Err(GoalError::NoSuchGoal),
            Entry::Occupied(o) => match o.get() {
                AsyncGoal {
                    status: GoalStatusEnum::Unknown,
                    ..
                } => {
                    let now = builtin_interfaces::Time::now();
                    let mut_o = o.into_mut();
                    mut_o.status = GoalStatusEnum::Accepted;
                    mut_o.accepted_time = Some(now);
                    self.publish_statuses().await;
                    self.actionserver.my_goal_server.send_response(
                        handle.req_id,
                        SendGoalResponse {
                            accepted: true,
                            stamp: now,
                        },
                    )?;
                    Ok(AcceptedGoalHandle {
                        inner: handle.inner,
                    })
                }
                AsyncGoal {
                    status: wrong_status,
                    ..
                } => {
                    log::error!(
                        "Tried to accept goal {:?} but status was {:?}, expected Unknown.",
                        handle.inner.goal_id,
                        wrong_status
                    );
                    Err(GoalError::WrongGoalState)
                }
            },
        }
    }

    /// Reject a received goal. Client will be notified of rejection.
    /// Server should not process the goal further.
    pub async fn reject_goal(
        &mut self,
        handle: NewGoalHandle<A::GoalType>,
    ) -> Result<(), GoalError<()>>
    where
        A::GoalType: 'static,
    {
        match self.goals.entry(handle.inner.goal_id) {
            Entry::Vacant(_) => Err(GoalError::NoSuchGoal),
            Entry::Occupied(o) => {
                match o.get() {
                    AsyncGoal {
                        status: GoalStatusEnum::Unknown,
                        ..
                    } => {
                        self.actionserver.my_goal_server.send_response(
                            handle.req_id,
                            SendGoalResponse {
                                accepted: false,
                                stamp: builtin_interfaces::Time::now(),
                            },
                        )?;
                        //o.into_mut().0 = GoalStatusEnum::Rejected; -- there is no such state
                        //self.publish_statuses().await; -- this is not reported
                        Ok(())
                    }
                    AsyncGoal {
                        status: wrong_status,
                        ..
                    } => {
                        log::error!(
                            "Tried to reject goal {:?} but status was {:?}, expected Unknown.",
                            handle.inner.goal_id,
                            wrong_status
                        );
                        Err(GoalError::WrongGoalState)
                    }
                }
            }
        }
    }

    /// Convert an accepted goal into a expecting goal, i.e. start the execution.
    /// Executing goal can publish feedback.
    pub async fn start_executing_goal(
        &mut self,
        handle: AcceptedGoalHandle<A::GoalType>,
    ) -> Result<ExecutingGoalHandle<A::GoalType>, GoalError<()>> {
        match self.goals.entry(handle.inner.goal_id) {
            Entry::Vacant(_) => Err(GoalError::NoSuchGoal),
            Entry::Occupied(o) => match o.get() {
                AsyncGoal {
                    status: GoalStatusEnum::Accepted,
                    ..
                } => {
                    o.into_mut().status = GoalStatusEnum::Executing;
                    self.publish_statuses().await;
                    Ok(ExecutingGoalHandle {
                        inner: handle.inner,
                    })
                }
                AsyncGoal {
                    status: wrong_status,
                    ..
                } => {
                    log::error!(
                        "Tried to execute goal {:?} but status was {:?}, expected Accepted.",
                        handle.inner.goal_id,
                        wrong_status
                    );
                    Err(GoalError::WrongGoalState)
                }
            },
        }
    }

    /// Publish feedback on how the execution is proceeding.
    pub async fn publish_feedback(
        &mut self,
        handle: ExecutingGoalHandle<A::GoalType>,
        feedback: A::FeedbackType,
    ) -> Result<(), GoalError<FeedbackMessage<A::FeedbackType>>> {
        match self.goals.entry(handle.inner.goal_id) {
            Entry::Vacant(_) => Err(GoalError::NoSuchGoal),
            Entry::Occupied(o) => match o.get() {
                AsyncGoal {
                    status: GoalStatusEnum::Executing,
                    ..
                } => {
                    self.actionserver
                        .send_feedback(handle.inner.goal_id, feedback)?;
                    Ok(())
                }
                AsyncGoal {
                    status: wrong_status,
                    ..
                } => {
                    log::error!(
            "Tried publish feedback on goal {:?} but status was {:?}, expected Executing.",
            handle.inner.goal_id, wrong_status
          );
                    Err(GoalError::WrongGoalState)
                }
            },
        }
    }

    /// Notify Client that a goal end state was reached and
    /// what was the result of the action.
    /// This async will not resolve until the action client has requested for the
    /// result, but the client should request the result as soon as server
    /// accepts the goal.
    // TODO: It is a bit silly that we have to supply a "result" even though
    // goal got canceled. But we have to send something in the ResultResponse.
    // And where does it say that result is not significant if cancelled or aborted?
    pub async fn send_result_response(
        &mut self,
        handle: ExecutingGoalHandle<A::GoalType>,
        result_status: GoalEndStatus,
        result: A::ResultType,
    ) -> Result<(), GoalError<()>>
    where
        A::ResultType: 'static,
    {
        // We translate from interface type to internal type to ensure that
        // the end status is an end status and not e.g. "Accepted".
        let result_status = match result_status {
            GoalEndStatus::Succeeded => GoalStatusEnum::Succeeded,
            GoalEndStatus::Aborted => GoalStatusEnum::Aborted,
            GoalEndStatus::Canceled => GoalStatusEnum::Canceled,
        };

        // First, we must get a result request.
        // It may already have been read or not.
        // We will read these into a buffer, because there may be requests for
        // other goals' results also.
        let req_id = match self.result_requests.get(&handle.inner.goal_id) {
            Some(req_id) => *req_id,
            None => {
                let res_reqs = self.actionserver.my_result_server.receive_request_stream();
                pin_mut!(res_reqs);
                loop {
                    // result request was not yet here. Keep receiving until we get it.
                    let (req_id, GetResultRequest { goal_id }) =
                        res_reqs.select_next_some().await?;
                    if goal_id == handle.inner.goal_id {
                        break req_id;
                    } else {
                        self.result_requests.insert(goal_id, req_id);
                        log::debug!(
                            "Got result request for goal_id={:?} req_id={:?}",
                            goal_id,
                            req_id
                        );
                        // and loop to wait for the next
                    }
                }
            }
        };

        match self.goals.entry(handle.inner.goal_id) {
            Entry::Vacant(_) => Err(GoalError::NoSuchGoal),
            Entry::Occupied(o) => {
                match o.get() {
                    // Accepted, executing, or canceling goal can be canceled or aborted
                    // TODO: Accepted goal cannot succeed, it must be executing before success.
                    AsyncGoal {
                        status: GoalStatusEnum::Accepted,
                        ..
                    }
                    | AsyncGoal {
                        status: GoalStatusEnum::Executing,
                        ..
                    }
                    | AsyncGoal {
                        status: GoalStatusEnum::Canceling,
                        ..
                    } => {
                        o.into_mut().status = result_status;
                        self.publish_statuses().await;
                        self.actionserver.send_result(
                            req_id,
                            GetResultResponse {
                                status: result_status,
                                result,
                            },
                        )?;
                        log::debug!(
                            "Send result for goal_id={:?}  req_id={:?}",
                            handle.inner.goal_id,
                            req_id
                        );
                        Ok(())
                    }
                    AsyncGoal {
                        status: wrong_status,
                        ..
                    } => {
                        log::error!(
                            "Tried to finish goal {:?} but status was {:?}.",
                            handle.inner.goal_id,
                            wrong_status
                        );
                        Err(GoalError::WrongGoalState)
                    }
                }
            }
        }
    }

    /// Abort goal execution, because action server has determined it
    /// cannot continue execution.
    pub async fn abort_executing_goal(
        &mut self,
        handle: ExecutingGoalHandle<A::GoalType>,
    ) -> Result<(), GoalError<()>> {
        self.abort_goal(handle.inner).await
    }

    /// Aborts an accepted goal.
    pub async fn abort_accepted_goal(
        &mut self,
        handle: AcceptedGoalHandle<A::GoalType>,
    ) -> Result<(), GoalError<()>> {
        self.abort_goal(handle.inner).await
    }

    async fn abort_goal(
        &mut self,
        handle: InnerGoalHandle<A::GoalType>,
    ) -> Result<(), GoalError<()>> {
        match self.goals.entry(handle.goal_id) {
            Entry::Vacant(_) => Err(GoalError::NoSuchGoal),
            Entry::Occupied(o) => match o.get() {
                AsyncGoal {
                    status: GoalStatusEnum::Accepted,
                    ..
                }
                | AsyncGoal {
                    status: GoalStatusEnum::Executing,
                    ..
                } => {
                    o.into_mut().status = GoalStatusEnum::Aborted;
                    self.publish_statuses().await;
                    Ok(())
                }
                AsyncGoal {
                    status: wrong_status,
                    ..
                } => {
                    log::error!(
            "Tried to abort goal {:?} but status was {:?}, expected Accepted or Executing. ",
            handle.goal_id, wrong_status
          );
                    Err(GoalError::WrongGoalState)
                }
            },
        }
    }

    /// Receive a set of cancel requests from the action client.
    /// The server should now respond either by accepting (some of) the
    /// cancel requests or rejecting all of them. The GoalIds that are requested
    /// to be cancelled can be currently at either accepted or executing state.
    pub async fn receive_cancel_request(&self) -> ReadResult<CancelHandle> {
        let (req_id, CancelGoalRequest { goal_info }) = self
            .actionserver
            .my_cancel_server
            .async_receive_request()
            .await?;

        #[allow(clippy::type_complexity)] // How would you refactor this type?
        let goal_filter: Box<dyn FnMut(&(&GoalId, &AsyncGoal<A>)) -> bool> = match goal_info {
            GoalInfo {
                goal_id: GoalId::ZERO,
                stamp: Time::ZERO,
            } => Box::new(|(_, _)| true), // cancel all goals

            GoalInfo {
                goal_id: GoalId::ZERO,
                stamp,
            } => Box::new(move |(_, ag)| ag.accepted_time.map(|at| at < stamp).unwrap_or(false)),

            GoalInfo {
                goal_id,
                stamp: Time::ZERO,
            } => Box::new(move |(g_id, _)| goal_id == **g_id),

            GoalInfo { goal_id, stamp } => Box::new(move |(g_id, ag)| {
                goal_id == **g_id || ag.accepted_time.map(move |at| at < stamp).unwrap_or(false)
            }),
        };

        // TODO:
        // Should check if the specified GoalId was unknown to us
        // or already terminated.
        // In those case outright send a negative response and not return to the
        // application.
        let cancel_handle = CancelHandle {
            req_id,
            goals: self
                .goals
                .iter()
                // only consider goals with status Executing or Accepted for Cancel
                .filter(|(_, async_goal)| {
                    async_goal.status == GoalStatusEnum::Executing
                        || async_goal.status == GoalStatusEnum::Accepted
                })
                // and then filter those that were specified by the cancel request
                .filter(goal_filter)
                .map(|p| *p.0)
                .collect(),
        };

        Ok(cancel_handle)
    }

    /// Respond to action client's cancel requests.
    /// The iterator of goals should list those GoalIds that will start canceling.
    /// For the other GoalIds, the cancel is not accepted and they do not change
    /// their state.
    pub async fn respond_to_cancel_requests(
        &mut self,
        cancel_handle: &CancelHandle,
        goals_to_cancel: impl Iterator<Item = GoalId>,
    ) -> WriteResult<(), ()> {
        let canceling_goals: Vec<GoalInfo> = goals_to_cancel
            .filter_map(|goal_id| {
                self.goals
                    .get(&goal_id)
                    .and_then(|AsyncGoal { accepted_time, .. }| {
                        accepted_time.map(|stamp| GoalInfo { goal_id, stamp })
                    })
            })
            .collect();

        for goal_info in &canceling_goals {
            self.goals
                .entry(goal_info.goal_id)
                .and_modify(|gg| gg.status = GoalStatusEnum::Canceling);
        }
        self.publish_statuses().await;

        let response = goal::CancelGoalResponse {
            return_code: if canceling_goals.is_empty() {
                goal::CancelGoalResponseEnum::Rejected
            } else {
                goal::CancelGoalResponseEnum::None // i.e. no error
            },
            goals_canceling: canceling_goals,
        };

        self.actionserver
            .my_cancel_server
            .async_send_response(cancel_handle.req_id, response)
            .await
    }

    // This function is private, because all status publishing happens automatically
    // via goal status changes.
    async fn publish_statuses(&self) {
        let goal_status_array = goal::GoalStatusArray {
            status_list: self
                .goals
                .iter()
                .map(
                    |(
                        goal_id,
                        AsyncGoal {
                            status,
                            accepted_time,
                            ..
                        },
                    )| goal::GoalStatus {
                        status: *status,
                        goal_info: GoalInfo {
                            goal_id: *goal_id,
                            stamp: accepted_time.unwrap_or(builtin_interfaces::Time::ZERO),
                        },
                    },
                )
                .collect(),
        };
        log::debug!(
            "Reporting statuses for {:?}",
            goal_status_array
                .status_list
                .iter()
                .map(|gs| gs.goal_info.goal_id)
        );
        self.actionserver
            .send_goal_statuses(goal_status_array)
            .unwrap_or_else(|e| log::error!("AsyncActionServer::publish_statuses: {:?}", e));
    }
}
