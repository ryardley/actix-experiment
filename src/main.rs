use actix::prelude::*;
use std::collections::HashMap;

#[derive(Message, Clone, Debug, PartialEq, Eq, Hash)]
#[rtype(result = "()")]
enum EnclaveEvent {
    ComputationRequested {
        e3_id: String,
        nodecount: u32,
        sortition_seed: u32,
    },
    KeyshareCreated {
        e3_id: String,
        keyshare: String,
    },
}

#[derive(Message, Clone, Debug)]
#[rtype(result = "Vec<EnclaveEvent>")]
struct GetLogs;

impl EnclaveEvent {
    pub fn event_type(&self) -> String {
        match self {
            EnclaveEvent::KeyshareCreated { .. } => "KeyshareCreated".to_string(),
            EnclaveEvent::ComputationRequested { .. } => "ComputationRequested".to_string(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Subscribe {
    event_type: String,
    listener: Recipient<EnclaveEvent>,
}

struct EventBus {
    listeners: HashMap<String, Vec<Recipient<EnclaveEvent>>>,
}

impl Actor for EventBus {
    type Context = Context<Self>;
}

impl EventBus {
    fn new() -> Self {
        EventBus {
            listeners: HashMap::new(),
        }
    }
}

impl Handler<Subscribe> for EventBus {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Context<Self>) {
        self.listeners
            .entry(msg.event_type)
            .or_insert_with(Vec::new)
            .push(msg.listener);
    }
}

impl Handler<EnclaveEvent> for EventBus {
    type Result = ();

    fn handle(&mut self, msg: EnclaveEvent, _: &mut Context<Self>) {
        if let Some(listeners) = self.listeners.get("*") {
            for listener in listeners {
                listener.do_send(msg.clone())
            }
        }

        if let Some(listeners) = self.listeners.get(&msg.event_type()) {
            for listener in listeners {
                listener.do_send(msg.clone())
            }
        }
    }
}

struct Listener {
    logs: Vec<EnclaveEvent>,
}
impl Listener {
    fn new() -> Self {
        Self { logs: vec![] }
    }
}

impl Actor for Listener {
    type Context = Context<Self>;
}

impl Handler<EnclaveEvent> for Listener {
    type Result = ();

    fn handle(&mut self, msg: EnclaveEvent, _: &mut Context<Self>) {
        println!("Listener received event: {:?}", msg);
        self.logs.push(msg);
    }
}
impl Handler<GetLogs> for Listener {
    type Result = Vec<EnclaveEvent>;

    fn handle(&mut self, _msg: GetLogs, _ctx: &mut Context<Self>) -> Self::Result {
        self.logs.iter().cloned().collect()
    }
}

#[actix_rt::main]
async fn main() {
    actix_rt::time::sleep(std::time::Duration::from_secs(1)).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix::test]
    async fn test_main() {
        // Create the EventBus actor
        let event_bus = EventBus::new().start();

        // Create a listener
        let listener = Listener::new().start();

        // Subscribe the listener to an event type
        event_bus
            .send(Subscribe {
                event_type: "*".to_string(),
                listener: listener.clone().recipient(),
            })
            .await
            .unwrap();

        // Send an event
        event_bus
            .send(EnclaveEvent::KeyshareCreated {
                e3_id: "123".to_string(),
                keyshare: "Hello".to_string(),
            })
            .await
            .unwrap();

        let logs = listener.send(GetLogs).await.unwrap();

        assert_eq!(
            logs,
            vec![EnclaveEvent::KeyshareCreated {
                e3_id: "123".to_string(),
                keyshare: "Hello".to_string(),
            }]
        );
    }
}
