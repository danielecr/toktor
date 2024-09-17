# Toktor : Tokio based Actor model implementation

!!!DISCLAIMER!!! this is a first port from another project, not warranty to work at all.

This package provides macros

- `actor_handler!({ par1: &str, par2: &str} => ActorObj, ActorName, MsgType)`;
- `toktor_new!(RequestsVisor, &par1, &par2)`
- `toktor_send!(s, msg)`

actor_handler! is a procedural macro, others are declarative macros.

## Example usage

An ActorObject is meant to receive a message and deal with it.
So, the first step is to define a MessageType by defining a struct or an enum.
Often a message contains an channel by which is possible to give feedback to the requester.
Example:

```
enum ReqVisorMsg {
    RegisterPending {
        //req: Request<IncomingBody>,
        req: RestMessage,
        respond_to: oneshot::Sender<(Receiver<FrontResponse>,String)>
    },
    FulfillPending {
        req_id: String,
        response: ForHttpResponse,
        respond_to: oneshot::Sender<bool>
        // the response is true if req_id match some unfulfilled message
        // it is false elsewise
    }
}
```

Then it comes the turn of the ActorObj, this is an kind of encapsulated resource, that
handle the message once each time. It is a (private) struct.
It has of course an new() method, the parameter list of `ActorObj::new()` is very important since
it is someway coupled with the `ActorHandler::new()`.
Other required methods are `async fn run(&mut self)` and `handle_message()`.
Of course the message must have a channel by which it is dispatched, and it is of type `mpsc`,
so that the actor can answer more users.

```
struct RequestsVisorObj {
    receiver: mpsc::Receiver<ReqVisorMsg>,
    conf: ServiceConf
}

impl RequestsVisorObj {
    pub fn new(receiver: mpsc::Receiver<ReqVisorMsg>, conf: &ServiceConf) -> Self {
        RequestsVisorActor {
            receiver,
            config: conf.clone()
        }
    }

    async fn run(&mut self) {
        while let Some(msg) = self.receiver.recv().await {
            self.handle_message(msg);
        }
    }

    fn handle_message(&mut self, msg: ReqVisorMsg) {
        match msg {
            ReqVisorMsg::RegisterPending { req, respond_to } => {
                // tokio::spawn(async move {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let uuid: String = uuid::Uuid::new_v4().to_string();
                let _ = respond_to.send((rx,uuid));
            }
            ReqVisorMsg::FulfillPending { req_id, response, respond_to } => {
                // .. impl
            }
        }
}

// this is the way the handler boilerplate is created

actor_handler!({pctl: &ProcessController, conf: &ServiceConf} => RequestsVisorObj, RequestsVisor, ReqVisorMsg);


impl RequestsVisor {
    pub fn push_fulfill(&self, req_id: &str, response: ForHttpResponse)-> tokio::sync::oneshot::Receiver<bool> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let msg = ReqVisorMsg::FulfillPending {
            req_id: req_id.to_string(),
            response,
            respond_to: tx
        };
        let s = self.clone();
        tokio::spawn(async move {
            match toktor_send!(s, msg).await {
                _ => println!()
            };
        });
        rx
    }
}
```

What really does:
> `actor_handler!({conf: &ServiceConf} => RequestsVisorObj, RequestsVisor, ReqVisorMsg);`
 
is the generation of the code:
```
#[derive(Clone)]
pub struct RequestsVisor {
    pub sender: ::tokio::sync::mpsc::Sender<Msg>,
}
impl RequestsVisor {
    pub fn new( conf: &ServiceConf ) -> RequestsVisor {
        let (sender, receiver) = ::tokio::sync::mpsc::channel(8);
        let mut actor = RequestsVisorObj::new(receiver, conf: &ServiceConf );
        ::tokio::spawn(async move { actor.run().await; });
        RequestsVisor {sender}
    }
}
```

For **Actor instantiation and usage** there are 2 convenient macros:

```
// create an actor handler
let visor = toktor_new!(RequestVisor, &conf);
```

and:

```
// send the message msg to actor handler s
toktor_send!(s, msg).await;
```

## TODO

This is a starting point, there is still some cut&paste code, like `fn run()` that is almost
the same everytime.

Anyway this is not meant to be simple or to hide the message mechanism, also handle() method
is not async by design, meaning that it must be called sequencially for each message,
this limitation can be surpassed by the use of `tokio::spawn(async ...)`,
but it is a deliberate choice.


## Credits

This implementation is inspired by Alice Ryhl blog post in https://ryhl.io/blog/actors-with-tokio/

