use crate::api;
use base64::{Engine as _, engine::general_purpose};
use leptos::logging::debug_log;
use leptos::prelude::*;
use uuid::Uuid;

/// Share popup component - asks user if they want to share
#[component]
pub fn SharePopup(
    show: ReadSignal<bool>,
    on_close: impl Fn() + 'static,
    on_share: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        {move || {
            if show.get() {
                view! {
                    <div
                        class="fixed bottom-6 right-6 bg-white rounded-lg shadow-lg border border-gray-200 p-4 z-40 animate-in slide-in-from-bottom-4 duration-300"
                        style="width: 280px;"
                    >
                        <div class="flex items-start justify-between mb-3">
                            <h4 class="font-semibold text-gray-800">"Share this work?"</h4>
                            <button
                                class="text-gray-400 hover:text-gray-600 transition-colors"
                                on:click=move |_| on_close()
                            >
                                <i class="fa-solid fa-xmark"></i>
                            </button>
                        </div>
                        <p class="text-sm text-gray-500 mb-4">
                            "Share to community for others to enjoy"
                        </p>
                        <div class="flex gap-3">
                            <button
                                class="flex-1 px-3 py-2 bg-gray-100 hover:bg-gray-200 text-gray-800 rounded-lg transition-colors text-sm font-medium"
                                on:click=move |_| on_close()
                            >
                                "Later"
                            </button>
                            <button
                                class="flex-1 px-3 py-2 bg-primary hover:bg-primary/90 text-white rounded-lg transition-colors text-sm font-medium"
                                on:click=move |_| on_share()
                            >
                                "Yes"
                            </button>
                        </div>
                    </div>
                }
                    .into_any()
            } else {
                view! { <div></div> }.into_any()
            }
        }}
    }
}

/// Full-screen share modal component
#[component]
pub fn ShareModal(
    show: ReadSignal<bool>,
    on_close: impl Fn() + 'static + Clone,
    on_create: impl Fn(String, String) + 'static + Clone,
    initial_title: impl Fn() -> String + 'static,
    initial_content: impl Fn() -> String + 'static,
    audio_data: ReadSignal<Vec<u8>>,
    voice_id: ReadSignal<String>,
) -> impl IntoView {
    let (title, set_title) = signal(String::new());
    let (content, set_content) = signal(String::new());
    let (is_submitting, set_is_submitting) = signal(false);

    // Update initial values when modal opens
    Effect::new(move |_| {
        if show.get() {
            set_title.set(initial_title());
            set_content.set(initial_content());
        }
    });

    let create_post_action = Action::new(move |(title_val, content_val): &(String, String)| {
        let title_val = title_val.clone();
        let content_val = content_val.clone();
        let audio_data_val = audio_data.get();
        let voice_id_val = voice_id.get();

        async move {
            // Create post info
            let post_id = Uuid::new_v4().to_string();

            // Encode audio data to base64
            let audio_b64 = general_purpose::STANDARD.encode(&audio_data_val);

            // Build metadata JSON
            let metadata = serde_json::json!({
                "description": content_val,
                "audio_data": audio_b64,
                "voice_meta_id": voice_id_val,
            })
            .to_string();

            let post_info = api::PostInfo {
                id: post_id,
                title: title_val,
                metadata,
            };

            api::create_post(post_info).await
        }
    });

    let handle_create = {
        let on_close = on_close.clone();
        let on_create = on_create.clone();
        move |_| {
            let title_val = title.get();
            let content_val = content.get();

            if title_val.trim().is_empty() || content_val.trim().is_empty() {
                return;
            }

            create_post_action.dispatch((title_val, content_val));
        }
    };

    // Handle action result
    Effect::new(move |_| {
        if let Some(result) = create_post_action.value().get() {
            match result {
                Ok(_) => {
                    debug_log!("Post created successfully");
                    on_create(String::new(), String::new());
                    on_close();
                }
                Err(e) => {
                    leptos::logging::error!("Failed to create post: {}", e);
                }
            }
        }
    });

    view! {
        {move || {
            if show.get() {
                let on_close_inner = on_close.clone();
                view! {
                    <div
                        class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4"
                        on:click=move |_| on_close_inner()
                    >
                        <div
                            class="bg-white rounded-xl shadow-2xl w-full max-w-2xl max-h-[90vh] flex flex-col"
                            on:click=move |e: web_sys::MouseEvent| e.stop_propagation()
                        >
                            // Header
                            <div class="flex justify-between items-center p-6 border-b border-gray-200">
                                <h2 class="text-2xl font-bold text-gray-800">"Create Share"</h2>
                                <button
                                    class="text-gray-400 hover:text-gray-600 transition-colors"
                                    on:click=move |_| on_close()
                                >
                                    <i class="fa-solid fa-xmark text-xl"></i>
                                </button>
                            </div>

                            // Content area (scrollable)
                            <div class="flex-1 overflow-y-auto p-6 space-y-6">
                                // Title input
                                <div>
                                    <label class="block text-sm font-semibold text-gray-700 mb-2">
                                        "Title"
                                    </label>
                                    <input
                                        type="text"
                                        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all"
                                        placeholder="Enter title..."
                                        prop:value=move || title.get()
                                        on:input=move |ev| set_title.set(event_target_value(&ev))
                                    />
                                </div>

                                // Content input
                                <div>
                                    <label class="block text-sm font-semibold text-gray-700 mb-2">
                                        "Description"
                                    </label>
                                    <textarea
                                        class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 focus:border-primary transition-all resize-none"
                                        rows="8"
                                        placeholder="Enter description..."
                                        prop:value=move || content.get()
                                        on:input=move |ev| set_content.set(event_target_value(&ev))
                                    ></textarea>
                                </div>

                                // Info box
                                <div class="bg-blue-50 border border-blue-200 rounded-lg p-3">
                                    <p class="text-xs text-blue-700">
                                        <i class="fa-solid fa-circle-info mr-2"></i>
                                        "After sharing, your work will be visible to all users"
                                    </p>
                                </div>
                            </div>

                            // Bottom action bar
                            <div class="flex justify-end gap-3 p-6 border-t border-gray-200 bg-gray-50">
                                <button
                                    class="px-6 py-2 bg-gray-200 hover:bg-gray-300 text-gray-800 rounded-lg transition-colors font-medium"
                                    on:click=move |_| on_close()
                                    disabled=move || create_post_action.pending().get()
                                >
                                    "Cancel"
                                </button>
                                <button
                                    class="px-6 py-2 bg-primary hover:bg-primary/90 text-white rounded-lg transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
                                    on:click=handle_create
                                    disabled=move || {
                                        create_post_action.pending().get()
                                            || title.get().trim().is_empty()
                                            || content.get().trim().is_empty()
                                    }
                                >
                                    {move || {
                                        if create_post_action.pending().get() {
                                            "Creating..."
                                        } else {
                                            "Create"
                                        }
                                    }}
                                </button>
                            </div>
                        </div>
                    </div>
                }
                    .into_any()
            } else {
                view! { <div></div> }.into_any()
            }
        }}
    }
}
