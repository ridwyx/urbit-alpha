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
        let metadata_channel = &mut self.ship.create_channel().ok()?;
        let invite_channel = &mut self.ship.create_channel().ok()?;
        let settings_channel = &mut self.ship.create_channel().ok()?;
        let btc_channel = &mut self.ship.create_channel().ok()?;

        // make channell
        // init wallet? oh duh just use /all and btc-wallet
        // get tx -hist hopefully
        // process payment hist

        channel.create_new_subscription("graph-store", "/updates").ok()?;
        metadata_channel.create_new_subscription("metadata-store", "/all").ok()?;
        invite_channel.create_new_subscription("invite-store", "/updates").ok()?;
        settings_channel.create_new_subscription("settings-store", "/all").ok()?;
        btc_channel.create_new_subscription("btc-wallet", "/all").ok()?;

        // Infinitely watch for new updates
        loop {
            channel.parse_event_messages();
            metadata_channel.parse_event_messages();
            invite_channel.parse_event_messages();
            btc_channel.parse_event_messages();

            let mut messages_to_send = vec![];
            let mut chats_to_join: Vec<ShipChat> = Vec::new();

            let graph_updates = &mut channel.find_subscription("graph-store", "/updates")?;
            let invite_updates = &mut invite_channel.find_subscription("invite-store", "/updates")?;
            let metadata_updates = &mut metadata_channel.find_subscription("metadata-store", "/all")?;
            let btc_updates = &mut btc_channel.find_subscription("btc-wallet", "/all")?;

            // Read all of the current SSE messages to find if any are for the chat
            // we are looking for.
            loop {
                let pop_invite = invite_updates.pop_message();
                let pop_metadata = metadata_updates.pop_message();
                let pop_message = graph_updates.pop_message();
                let btc_message = btc_updates.pop_message();

                println!("{:?}", btc_message);
                // Process invitations to new groups
                if let Some(invite) = &pop_invite {
                    let invite_result = self.invite_accept(invite);
                    match invite_result {
                        Ok(true) => println!("Successfully accepted invite."),
                        Ok(false) => (), // Ignore when invite-store sends a message that confirms we accepted the invite
                        Err(e) => println!("There was an error accepting the invite: {}", e)
                    }
                }
                // Get any newly created chats in our groups
                if let Some(metadata) = &pop_metadata {
                    chats_to_join = self.get_chats_to_join(metadata);
                }
                // Process new messages, determine if we should reply
                if let Some(message) = &pop_message {
                    messages_to_send = self.get_messages_to_send(message);
                }
                // If no messages left, stop
                // TODO should we only break if all three channels have no messages left?
                if matches!(&pop_message, None) && matches!(&pop_invite, None) && matches!(&pop_metadata, None) {
                    break;
                }
            }

            // Join newly added chats
            for chat in chats_to_join.iter() {
                println!("Attempting to join {} {}", chat.ship_name, chat.chat_name);
                let spider_data = self.build_chat_join_json(chat.ship_name.clone(), chat.chat_name.clone());
                let spider = channel.spider(
                    "landscape",
                    "json",
                    "graph-view-action/graph-join",
                    &spider_data,
                );
                thread::sleep(Duration::new(0, 500000000));

                if let Ok(spider_response) = spider {
                    println!("Actually joined chat {} on ship {}", chat.chat_name, chat.ship_name);
                }
            }

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

    // Returns the bot's reply to a message if the message is a command.
    fn get_messages_to_send(&self, message: &str) -> Vec<MessagePayload> {
        let mut messages_to_send = vec![];
        // Parse it to json
        if let Ok(json) = json::parse(message) {
            let origin_ship_chat = self.get_ship_chat_from_resource_json(&json);

            // Otherwise, parse json to a `Node`
            if let Ok(node) = Node::from_graph_update_json(&json) {
                // If the message is posted by the Chatbot ship, ignore
                if node.author == self.ship.ship_name {
                    return messages_to_send;
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
                    messages_to_send.push(MessagePayload {
                        message: message,
                        ship_chat: origin_ship_chat,
                    });
                } else {
                    println!("Message ignored.")
                }
            }
        }
        messages_to_send
    }
   

    // Accept an invite from a third party ship/chat
    // Return Ok(true) if invite was accepted
    // Return Ok(false) if we got a message from invite-store that wasn't necessarily the invite (this happens sometimes)
    pub fn invite_accept(&self, invite_message: &str) -> Result<bool, urbit_http_api::UrbitAPIError> {
        let invite_message_json = json::parse(invite_message).unwrap();
        if invite_message_json["invite-update"]["invite"].is_null() {
            return Ok(false);
        }
        let poke_channel = &mut self.ship.create_channel().unwrap();
        let ship = invite_message_json["invite-update"]["invite"]["invite"]["resource"]["ship"].clone().to_string();
        let name = invite_message_json["invite-update"]["invite"]["invite"]["resource"]["name"].clone().to_string();
        println!("Got an invite from group {} on ship {}. Raw JSON: {}", name, ship, invite_message_json);
        let poke = poke_channel.poke(
            "group-view",
            "group-view-action",
            &self.build_invite_accept_json(ship, name)
        );
        thread::sleep(Duration::new(0, 500000000));
        match poke {
            Ok(_) => Ok(true),
            Err(e) => Err(e)
        }
    }

    pub fn build_invite_accept_json(&self, ship: String, name: String) -> JsonValue {
        let mut json_object = JsonValue::new_object();
        json_object["join"] = JsonValue::new_object();
        json_object["join"]["resource"] = JsonValue::new_object();
        json_object["join"]["resource"]["ship"] = JsonValue::String(format!("~{}", ship.clone()));
        json_object["join"]["resource"]["name"] = JsonValue::String(name.clone().into());
        json_object["join"]["ship"] = JsonValue::String(format!("~{}", ship.clone()));
        json_object["join"]["app"] = JsonValue::String("groups".to_string());
        json_object["join"]["autojoin"] = JsonValue::Boolean(true);
        json_object["join"]["shareContact"] = JsonValue::Boolean(true);
        json_object
    }

    pub fn build_chat_join_json(&self, ship: String, chat: String) -> JsonValue {
        let mut json_object = JsonValue::new_object();
        json_object["join"] = JsonValue::new_object();
        json_object["join"] = JsonValue::new_object();
        json_object["join"]["resource"] = JsonValue::new_object();
        json_object["join"]["resource"]["ship"] = JsonValue::String(ship.clone().into());
        json_object["join"]["resource"]["name"] = JsonValue::String(chat.clone().into());
        json_object["join"]["ship"] = JsonValue::String(ship.clone().into());
        json_object
    }

    // Assembles list of chats to join.
    pub fn get_chats_to_join(&self, metadata_update: &str) -> Vec<ShipChat> {
        let mut chats_to_join: Vec<ShipChat> = Vec::new();
        let update_result: serde_json::Value = serde_json::from_str(metadata_update).unwrap();
        // Reacts when new chats are created
        if let Some(new_chat_update) = update_result["metadata-update"]["add"].as_object() {
            if new_chat_update["app-name"] == "graph" && new_chat_update["resource"].is_string() {
                let chat = self.chat_id_from_resource(new_chat_update["resource"].as_str().unwrap());
                println!("Joined Chat: {}", chat.chat_name);
                chats_to_join.push(chat);
            }
        }
        // Handles the case where new chats are created while the bot is offline.
        // This does not scale. A later version will not attempt to rejoin chats the bot already knows about, just new ones.
        if let Some(associations_update) = update_result["metadata-update"]["associations"].as_object() {
            for (_, value) in associations_update {
                if value["app-name"] == "graph" {
                    let chat = self.chat_id_from_resource(value["resource"].as_str().unwrap());
                    println!("In Chat: {}", chat.chat_name);
                    chats_to_join.push(chat);
                }
            }
        }
        // TODO: remove chat from our persistent store
        if let Some(removed_from_group_update) = update_result["metadata-update"]["remove"].as_object() {
            println!("Removed from Chat: {:?}", removed_from_group_update);
        }
        chats_to_join
    }

    fn get_ship_chat_from_resource_json(&self, resource_json: &JsonValue) -> ShipChat {
        let resource = resource_json["graph-update"]["add-nodes"]["resource"].clone();
        return ShipChat {
            ship_name: format!("~{}", resource["ship"]),
            chat_name: format!("{}", resource["name"]),
        };
    }

    fn chat_id_from_resource(&self, resource: &str) -> ShipChat {
        let splitted_value = resource.split("/");
        return ShipChat {
            ship_name: splitted_value.clone().collect::<Vec<&str>>()[2].to_string(),
            chat_name: splitted_value.clone().last().unwrap().to_string(),
        };
    }

    /// Deprecated: Urbit Alpha responds to all commands in all chats of which it is a member.
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


#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    fn respond_to_message(authored_message: AuthoredMessage) -> Option<Message> {
        None
    }

    #[test]
    fn test_build_invite_accept_json() {
        let shipchats: Vec<ShipChat> = Vec::new();
        let bot = Chatbot::new_with_local_config(respond_to_message, shipchats);
        let mut jsonObj = bot.build_invite_accept_json("dozzod-dozzod".to_string(), "groupname".to_string());
        // JSON for invite accept needs ~ in front of ship name
        assert_eq!(Some(jsonObj["join"]["resource"]["ship"].as_str().unwrap().get(0..1)), "~");
        // assert_eq!(jsonObj["join"]["ship"].as_str().unwrap().get(0..1), "~");
    }

}
