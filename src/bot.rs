use json::JsonValue;
use std::thread;
use std::time::Duration;
use urbit_http_api::{default_cli_ship_interface_setup, Node, NodeContents, ShipInterface};
pub use urbit_http_api::{AuthoredMessage, Message};

pub struct ShipChat {
    pub(crate) ship_name: String,
    pub(crate) chat_name: String,
}

struct MessagePayload {
    message: NodeContents,
    ship_chat: ShipChat,
}

/// This struct represents a chatbot that is connected to a given `ship`,
/// is watching/posting to a specific `chat_ship`/`chat_name`
/// and is using the function `respond_to_message` to process any messages
/// which are posted in said chat.
pub struct Chatbot {
    /// `respond_to_message` is a function defined by the user of this framework.
    /// This function receives any messages that get posted to the connected chat,
    /// and if the function returns `Some(message)`, then `message` is posted to the
    /// chat as a response. If it returns `None`, then no message is posted.
    respond_to_message: fn(AuthoredMessage) -> Option<Message>,
    ship: ShipInterface,
    ship_chats: Vec<ShipChat>,
}

impl Chatbot {
    /// Create a new `Chatbot` with a manually provided `ShipInterface`
    pub fn new(
        respond_to_message: fn(AuthoredMessage) -> Option<Message>,
        ship: ShipInterface,
        ship_chats: Vec<ShipChat>,
    ) -> Self {
        Chatbot {
            respond_to_message: respond_to_message,
            ship: ship,
            ship_chats: ship_chats,
        }
    }

    /// Create a new `Chatbot` with a `ShipInterface` derived automatically
    /// from a local config file. If the config file does not exist, the
    /// `Chatbot` will create the config file, exit, and prompt the user to
    /// fill it out.
    pub fn new_with_local_config(
        respond_to_message: fn(AuthoredMessage) -> Option<Message>,
        ship_chats: Vec<ShipChat>,
    ) -> Self {
        let ship = default_cli_ship_interface_setup();
        Self::new(respond_to_message, ship, ship_chats)
    }

    /// Run the `Chatbot`
    pub fn run(&self) -> Option<()> {
        println!("=======================================\nChatbot Powered By The Urbit Chatbot Framework\n=======================================");
        // Create a `Subscription`
        let channel = &mut self.ship.create_channel().ok()?;
        // Subscribe to all graph-store updates
        channel
            .create_new_subscription("graph-store", "/updates")
            .ok()?;

        channel
            .create_new_subscription("hark-store", "/updates")
            .ok()?;

        // Infinitely watch for new graph store updates
        loop {
            channel.parse_event_messages();
            let graph_updates = &mut channel.find_subscription("graph-store", "/updates")?;
            let mut messages_to_send = vec![];

            // Read all of the current SSE messages to find if any are for the chat
            // we are looking for.
            loop {
                let pop_res = graph_updates.pop_message();
                // Acquire the message
                if let Some(mess) = &pop_res {
                    // Parse it to json
                    if let Ok(json) = json::parse(mess) {
                        // println!("{:?}", &json);

                        // If the graph-store node update is not for the chat the `Chatbot`
                        // is watching, then continue to next message.
                        if !self.check_resource_json(&json) {
                            continue;
                        }

                        let origin_ship_chat = self.get_ship_chat_from_resource_json(&json);

                        // Otherwise, parse json to a `Node`
                        if let Ok(node) = Node::from_graph_update_json(&json) {
                            // If the message is posted by the Chatbot ship, ignore
                            // if node.author == self.ship.ship_name
                            if node.author == self.ship.ship_name {
                                continue;
                            }

                            // Else parse it as an `AuthoredMessage`
                            let authored_message = AuthoredMessage::new(
                                &node.author,
                                &node.contents,
                                &node.time_sent_formatted(),
                                &node.index,
                            );
                            // If the Chatbot intends to respond to the provided message
                            if let Some(message) = (self.respond_to_message)(authored_message) {
                                println!("Replied to message.");
                                // messages_to_send.push(message);
                                messages_to_send.push(MessagePayload {
                                    message: message,
                                    ship_chat: origin_ship_chat,
                                });
                            } else {
                                println!("Message ignored.")
                            }
                        }
                    }
                }
                // If no messages left, stop
                if let None = &pop_res {
                    break;
                }
            }

            // for ship_chat in &self.ship_chats {
            //     // TODO: If not subscribed, subscribe - this doesn't work yet - maybe Landscape update needed?
            //     let chat_receiver = channel
            //         .chat()
            //         .subscribe_to_chat(&ship_chat.ship_name, &ship_chat.chat_name);

            //     if let Ok(rec) = &chat_receiver {
            //         // println!("{:#?}", rec);
            //     }
            // }

            // Send each response message that was returned by the `respond_to_message`
            // function. This is separated until after done parsing messages due to mutable borrows.
            for message in messages_to_send {
                channel
                    .chat()
                    .send_chat_message(
                        &message.ship_chat.ship_name,
                        &message.ship_chat.chat_name,
                        &message.message,
                    )
                    .ok();
            }
            thread::sleep(Duration::new(0, 500000000));
        }
    }

    fn get_ship_chat_from_resource_json(&self, resource_json: &JsonValue) -> ShipChat {
        let resource = resource_json["graph-update"]["add-nodes"]["resource"].clone();

        println!(
            "Processing message on ~{} {}",
            resource["ship"], resource["name"]
        );

        return ShipChat {
            ship_name: format!("~{}", resource["ship"]),
            chat_name: format!("{}", resource["name"]),
        };
    }

    /// Checks whether the resource json matches the chat_name & chat_ship
    /// that this `Chatbot` is interacting with
    fn check_resource_json(&self, resource_json: &JsonValue) -> bool {
        // let resource = resource_json["graph-update"]["add-nodes"]["resource"].clone();
        // let chat_name = format!("{}", resource["name"]);
        // let chat_ship = format!("~{}", resource["ship"]);
        // if chat_name == self.chat_name && chat_ship == self.chat_ship {
        //     return true;
        // }

        // TODO: Add checking if the message is coming from one of the enabled ship/chats combos

        true
    }
}
