use crate::url_sharing::UrlSharing;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ShareButtonProps {
    pub program_name: String,
    pub program_code: String,
    pub is_enabled: bool,
}

pub enum ShareButtonMsg {
    GenerateShareUrl,
    CopyToClipboard(String),
    ShowSuccess,
    ShowError(String),
    ClearMessage,
}

pub struct ShareButton {
    share_url: Option<String>,
    message: Option<String>,
    is_success: bool,
    is_generating: bool,
    clear_timeout: Option<gloo_timers::callback::Timeout>,
}

impl Component for ShareButton {
    type Message = ShareButtonMsg;
    type Properties = ShareButtonProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            share_url: None,
            message: None,
            is_success: false,
            is_generating: false,
            clear_timeout: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ShareButtonMsg::GenerateShareUrl => {
                if !ctx.props().is_enabled {
                    self.message = Some("Please fix program errors before sharing".to_string());
                    self.is_success = false;
                    return true;
                }

                self.is_generating = true;
                self.message = None;

                let name = ctx.props().program_name.clone();
                let code = ctx.props().program_code.clone();
                let link = ctx.link().clone();

                spawn_local(async move {
                    match UrlSharing::generate_share_url(&name, &code) {
                        Ok(url) => {
                            link.send_message(ShareButtonMsg::CopyToClipboard(url));
                        }
                        Err(e) => {
                            link.send_message(ShareButtonMsg::ShowError(format!(
                                "Failed to generate share URL: {}",
                                e
                            )));
                        }
                    }
                });

                true
            }
            ShareButtonMsg::CopyToClipboard(url) => {
                self.is_generating = false;
                self.share_url = Some(url.clone());

                // Try to copy to clipboard
                let link = ctx.link().clone();
                spawn_local(async move {
                    match UrlSharing::copy_to_clipboard(&url) {
                        Ok(_) => {
                            link.send_message(ShareButtonMsg::ShowSuccess);
                            // Clear the URL display after a delay
                            let link_clone = link.clone();
                            gloo_timers::future::TimeoutFuture::new(3000).await;
                            link_clone.send_message(ShareButtonMsg::ClearMessage);
                        }
                        Err(e) => {
                            link.send_message(ShareButtonMsg::ShowError(format!(
                                "Failed to copy to clipboard: {}",
                                e
                            )));
                        }
                    }
                });

                true
            }
            ShareButtonMsg::ShowSuccess => {
                self.message = Some("Share URL copied to clipboard!".to_string());
                self.is_success = true;

                // Cancel any existing timeout
                if let Some(timeout) = self.clear_timeout.take() {
                    timeout.cancel();
                }

                // Clear message after 3 seconds
                let link = ctx.link().clone();
                self.clear_timeout = Some(gloo_timers::callback::Timeout::new(3000, move || {
                    link.send_message(ShareButtonMsg::ClearMessage);
                }));

                true
            }
            ShareButtonMsg::ShowError(error) => {
                self.message = Some(error);
                self.is_success = false;

                // Cancel any existing timeout
                if let Some(timeout) = self.clear_timeout.take() {
                    timeout.cancel();
                }

                // Clear message after 5 seconds
                let link = ctx.link().clone();
                self.clear_timeout = Some(gloo_timers::callback::Timeout::new(5000, move || {
                    link.send_message(ShareButtonMsg::ClearMessage);
                }));

                true
            }
            ShareButtonMsg::ClearMessage => {
                self.message = None;
                self.is_success = false;
                self.clear_timeout = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();
        let props = ctx.props();

        let on_share_click = link.callback(|_| ShareButtonMsg::GenerateShareUrl);

        let button_class = if props.is_enabled {
            "btn btn-primary btn-sm"
        } else {
            "btn btn-disabled btn-sm"
        };

        html! {
            <div class="share-section">
                <div class="share-controls">
                    <button
                        class={button_class}
                        onclick={on_share_click}
                        disabled={self.is_generating || !props.is_enabled}
                        title={if props.is_enabled { "Generate shareable link" } else { "Fix program errors to enable sharing" }}
                    >
                        {if self.is_generating {
                            html! { <>
                                <span class="loading loading-spinner loading-xs"></span>
                                {"Generating..."}
                            </> }
                        } else if self.is_success {
                            html! { <>
                                <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7"></path>
                                </svg>
                                {"Copied!"}
                            </> }
                        } else {
                            html! { <>
                                <svg fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.367 2.684 3 3 0 00-5.367-2.684z"></path>
                                </svg>
                                {"Share"}
                            </> }
                        }}
                    </button>

                    {html! {}}
                </div>

                {html! {}}
            </div>
        }
    }
}
