use json::{object, JsonValue};
use serde_json::Value;
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
        //let metadata_channel = &mut self.ship.create_channel().ok()?;

        let invite_channel = &mut self.ship.create_channel().ok()?;
        let poke_channel = &mut self.ship.create_channel().ok()?;

        // Subscribe to all graph-store updates
        channel
            .create_new_subscription("graph-store", "/updates")
            .ok()?;
/* 
        metadata_channel
            .create_new_subscription("metadata-store", "/all")
            .ok()?; */
        invite_channel
            .create_new_subscription("invite-store", "/updates")
            .ok()?;

        // Infinitely watch for new graph store updates
        loop {
            channel.parse_event_messages();
            //metadata_channel.parse_event_messages();
            invite_channel.parse_event_messages();

            let mut messages_to_send = vec![];
            let mut chats_to_join: Vec<ShipChat> = Vec::new();

            let graph_updates = &mut channel.find_subscription("graph-store", "/updates")?;
            let invite_updates = &mut invite_channel.find_subscription("invite-store", "/updates")?;
            // let metadata_updates = &mut metadata_channel.find_subscription("metadata-store", "/all")?;
            let pop_invite = invite_updates.pop_message();
            if let Some(invite) = &pop_invite {
                // accept invite, join all chats
                println!("got invite: {}", invite);
                let invite_result = json::parse(invite).unwrap();
                let nested = &invite_result["invite-update"]["invite"]["invite"];
                let json_string = format!(
                    "{{\"join\":{{\"app\": \"groups\", \"autojoin\": true, \"shareContact\": true, \"resource\":{{\"ship\":\"~{ship}\",\"name\":\"{chat}\"}},\"ship\":\"~{ship}\"}}}}",
                    ship = nested["resource"]["ship"], chat = nested["resource"]["name"]
                );
                println!("json string: {}", json_string);
                let poke_data = json::parse(&json_string).ok().unwrap();
                let poke = poke_channel.poke(
                    "group-view",
                    "group-view-action",
                    &poke_data
                );
                // send ack
                let mut body = json::parse(r#"[]"#).unwrap();
                body[0] = object! {
                        "event-id": invite_updates.creation_id,
                        "action": "ack"
                };

                println!("this is ack body: {:?}", body);
        
                // Make the put request for the poke
                let ack = self.ship.send_put_request(&poke_channel.url, &body);

                thread::sleep(Duration::new(0, 500000000));

                if let Ok(poke_res) = poke {
                    println!("accepted invite, response was {:?}", poke_res);
                }
                if let Ok(ack_res) = ack {
                    println!("sent ack, response was {:?}", ack_res);
                }
            }
            // Read all of the current SSE messages to find if any are for the chat
            // we are looking for.
            loop {
                let pop_res = graph_updates.pop_message();
            /*
                let pop_update = metadata_updates.pop_message();

                if let Some(update) = &pop_update {
                    let update_result: serde_json::Value = serde_json::from_str(update).unwrap();
                    // On first run, checks for all available chats
                    if let Some(associations_update) =
                        update_result["metadata-update"]["associations"].as_object()
                    {
                        for (_, value) in associations_update {
                            if value["app-name"] == "graph" {
                                let chat = self.chat_id_from_resource(value);
                                println!("In Chat: {}", chat.chat_name);
                                chats_to_join.push(chat);
                            }
                        }
                    }

                    // what is initial-group?
                    if let Some(joined_group_update) = update_result["metadata-update"]
                        ["initial-group"]["associations"]
                        .as_object()
                    {
                        for (_, value) in joined_group_update {
                            if value["app-name"] == "graph" {
                                let chat = self.chat_id_from_resource(value);
                                println!("Joined Chat: {}", chat.chat_name);
                                chats_to_join.push(chat);
                            }
                        }
                    }

                    if let Some(removed_from_group_update) =
                        update_result["metadata-update"]["remove"].as_object()
                    {
                        println!("Removed from Chat: {:?}", removed_from_group_update);
                    }
                }
                */
                // Acquire the message
                if let Some(mess) = &pop_res {
                    // Parse it to json
                    if let Ok(json) = json::parse(mess) {
                        println!("{:?}", &json);

                        // If the graph-store node update is not for the chat the `Chatbot`
                        // is watching, then continue to next message.
/*                         if !self.check_resource_json(&json) {
                            continue;
                        } */

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
/*
            // Join newly added chats
            chats_to_join.retain(|chat| {
                let json_string = format!(
                    "{{\"join\":{{\"resource\":{{\"ship\":\"{ship}\",\"name\":\"{chat}\"}},\"ship\":\"{ship}\"}}}}",
                    ship = chat.ship_name, chat = chat.chat_name
                );
                let spider_data = json::parse(&json_string).ok().unwrap();
                let spider = channel.spider(
                    "landscape",
                    "json",
                    "graph-view-action/graph-join",
                    &spider_data,
                );

                thread::sleep(Duration::new(0, 500000000));

                if let Ok(spider_response) = spider {
                    println!("Actually joined chat: {:?}", spider_response);

                    // Send welcome message
                    channel
                        .chat()
                        .send_chat_message(
                            &chat.ship_name,
                            &chat.chat_name,
                            &Message::new().add_text(
                                "Urbit Alpha online.\n
                                Type `c <trading_pair> <timeframe>` to get the corresponding chart.\n
                                You can look up any trading pair and timeframe supported by TradingView.\n
                                Example: `c ethusd 4h`",
                            ),
                        )
                        .ok();

                    return false; // remove from chats_to_join
                } else {
                    return true; // keep in chats_to_join
                }
            });
*/
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

    fn chat_id_from_resource(&self, value: &Value) -> ShipChat {
        let resource = value["resource"].clone().to_string();
        let splitted_value = resource.split("/");

        let ship_id = splitted_value.clone().collect::<Vec<&str>>()[2].to_string();

        let mut chat_id = splitted_value.clone().last().unwrap().to_string();
        chat_id.pop(); // remove trailing quote

        return ShipChat {
            ship_name: ship_id,
            chat_name: chat_id,
        };
    }

    /// Checks whether the resource json matches one of the chat_name & chat_ship pairs
    /// that this `Chatbot` is interacting with
    fn check_resource_json(&self, resource_json: &JsonValue) -> bool {
        let resource = resource_json["graph-update"]["add-nodes"]["resource"].clone();
        let chat_name = format!("{}", resource["name"]);
        let chat_ship = format!("~{}", resource["ship"]);
        for ship_chat in &self.ship_chats {
            if chat_name == ship_chat.chat_name && chat_ship == ship_chat.ship_name {
                return true;
            }
        }
        false
    }
}
