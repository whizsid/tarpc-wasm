use crate::rpc_client::build_client;

use log::{info, Level};

use tarpc::context;
use rpc::WorldClient;

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;

pub mod rpc_client;

#[derive(Clone, Debug)]
pub struct Model {
    link: yew::html::Scope<Model>,
    delay: u64,
    delay_result: String,
    client: Rc<RefCell<Option<WorldClient>>>,
    echo_value: String,
    echo_result: String,
    connected: bool,
}

pub enum Msg {
    Connect,
    Connected,
    Ping,
    UpdateEcho(InputEvent),
    UpdateDelay(InputEvent),
    UpdateEchoResult(String),
    UpdateDelayResult(String),
    Echo,
    Delay,
    Redraw,
}

impl Model {
    fn connect(&mut self) {
        info!("Attemping to connect");
        let client_ptr = self.client.clone();
        let link = self.link.clone();
        info!("Connecting");
        spawn_local(async move {
            let transport = build_client();
            if let Ok(trans) = transport.await {
                info!("Connected");
                let config = tarpc::client::Config::default();
                let client = WorldClient::new(config, trans);
                let dispatch = client
                    .dispatch;
                info!("Spawning Dispatch");
                spawn_local(async move {dispatch.await.unwrap();});

                //Store the client.
                client_ptr.replace(Some(client.client));

                //Force the dom view to refresh to update the Connected status.
                link.send_message(Msg::Connected);
            }
        });
    }
    fn ping(&self) {
        if self.connected {
            let client = self.client.clone();
            let fut = async move {
                if let Some(ref mut client) = *client.borrow_mut() {
                    let result = client.ping(context::current()).await.unwrap();
                    if let Ok(msg) = result {
                        info!("Ping success: Results {}", msg);
                    }
                }
            };
            spawn_local(fut);
        }
    }

    fn echo(&self, value: String) {
        if self.connected {
            let client = self.client.clone();
            let link = self.link.clone();
            let fut = async move {
                if let Some(ref mut client) = *client.borrow_mut() {
                    let result = client.echo(context::current(), value).await.unwrap();
                    if let Ok(msg) = result {
                        info!("Echo Success: Results {}", msg);
                        link.send_message(Msg::UpdateEchoResult(msg));
                    }
                }
            };
            spawn_local(fut);
        }
    }

    fn delay(&self, delay: u64) {
        if self.connected {
            let client = self.client.clone();
            let link = self.link.clone();
            let fut = async move {
                if let Some(ref mut client) = *client.borrow_mut() {
                    let result = client.delay(context::current(), delay).await.unwrap();
                    if let Ok(msg) = result {
                        info!("Delayed Success: Results {}", msg);
                        link.send_message(Msg::UpdateDelayResult(msg));
                    } else {
                        link.send_message(Msg::UpdateDelayResult(format!("Delay failed {}", delay)))
                    }
                }
            };
            spawn_local(fut);
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self {
            link: ctx.link().clone(),
            client: Rc::new(RefCell::new(None)),
            delay: 30,
            delay_result: "Type number in input and press Delay".into(),
            echo_value: "".into(),
            echo_result: "Type string in input and press Echo".into(),
            connected: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Connect => self.connect(),
            Msg::Ping => self.ping(),
            Msg::UpdateEcho(e) => {
                let target:HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                self.echo_value = target.value();
            },
            Msg::UpdateDelay(e) => {
                let target:HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                self.delay = target.value().parse().unwrap();
            },
            Msg::UpdateDelayResult(result) => {
                info!("Updating the delay result");
                self.delay_result = result.clone();
            },
            Msg::Echo => self.echo(self.echo_value.clone()),
            Msg::Delay => self.delay(self.delay),
            Msg::Redraw => (),
            Msg::UpdateEchoResult(result) => {
                info!("Updating the echo result");
                self.echo_result = result.clone();
            }
            Msg::Connected => self.connected = true,
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let echo_result = self.echo_result.clone();
        html! {
            <div>
                <button onclick={ctx.link().callback(|_| Msg::Connect)}>{ "Connect" }</button>
                <button onclick={ctx.link().callback(|_| Msg::Ping)}>{ "Ping" }</button>
                <div>
                    <input
                        type = "text"
                        placeholder="Echo String"
                        value={self.echo_value.clone()}
                        oninput={ctx.link().callback(Msg::UpdateEcho)}
                    />
                    <button onclick={ctx.link().callback(|_| Msg::Echo)}> { "Echo"} </button>
                    <div>{"Echoed Result: "}{echo_result} </div>
                </div>
                <div>
                    <input
                        type = "number"
                        placeholder="Delay(s)"
                        value={format!("{}",self.delay)}
                        oninput={ctx.link().callback(Msg::UpdateDelay)}
                    />
                    <button onclick={ctx.link().callback(|_| Msg::Delay)}> { "Delay"} </button>
                    <div>{"Delayed Result: "}{self.delay_result.clone()} </div>
                </div>
                <div>
                {"Connected: "}{
                    if self.connected {
                        "True"
                    }else {
                        "False"
                    }
                }
                </div>
            </div>
        }
    }
}

fn main() {
    console_log::init_with_level(Level::Debug).unwrap();
    yew::Renderer::<Model>::new().render();
}
