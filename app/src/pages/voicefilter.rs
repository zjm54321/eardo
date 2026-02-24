use crate::api::post::{action_voice_meta_post, get_voice_meta_post_comments, list_voice_meta_post, Action, PostStatus, VoiceMetaPost, VoiceMetaPostID};
use crate::api::user::UserID;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_navigate;
use std::str::FromStr;

// Server functions for interactions
#[server]
pub async fn toggle_meta_post_like(post_id: String) -> Result<(), ServerFnError> {
    let id = VoiceMetaPostID::from_str(&post_id).map_err(|e| ServerFnError::new(e.to_string()))?;
    action_voice_meta_post(id, Action::LikeDislike).await
}

#[server]
pub async fn add_meta_post_comment(post_id: String, content: String) -> Result<(), ServerFnError> {
    let id = VoiceMetaPostID::from_str(&post_id).map_err(|e| ServerFnError::new(e.to_string()))?;
    action_voice_meta_post(id, Action::Comment(content)).await
}

#[server]
pub async fn get_meta_post_comments(post_id: String) -> Result<Vec<(String, String)>, ServerFnError> {
    let id = VoiceMetaPostID::from_str(&post_id).map_err(|e| ServerFnError::new(e.to_string()))?;
    let comments = get_voice_meta_post_comments(id).await?;
    Ok(comments.into_iter().map(|(user_id, content)| (user_id.to_string(), content)).collect())
}


// 用于显示的滤镜结构
#[derive(Debug, Clone)]
struct DisplayFilter {
    id: String,
    name: String,
    description: String,
    base_model_id: String,
    likes_count: i32,
    comments_count: i32,
    author: String,
    author_id: UserID,
    is_official: bool,
    is_liked: bool,
}

impl DisplayFilter {
    fn from_voice_meta(meta: VoiceMetaPost) -> Self {
        DisplayFilter {
            id: meta.id.to_string(),
            name: meta.title,
            description: meta.content,
            base_model_id: meta.meta_id.to_string(),
            likes_count: meta.likes_count,
            comments_count: meta.comments_count,
            author: meta.author.to_string(),
            author_id: meta.author,
            is_official: matches!(meta.status, PostStatus::Recommended),
            is_liked: meta.is_liked_by_current_user,
        }
    }
}

#[component]
pub fn VoiceFilterPage() -> impl IntoView {
    // 获取数据资源
    let recommended_filters_resource = Resource::new(
        || (),
        |_| list_voice_meta_post(Some(PostStatus::Recommended), 20),
    );

    let normal_filters_resource =
        Resource::new(|| (), |_| list_voice_meta_post(Some(PostStatus::Normal), 9));

    // 导航 hook
    let navigate = use_navigate();

    // 搜索词
    let (search, set_search) = signal(String::new());

    // 处理“使用滤镜”点击
    let apply_filter = move |filter: DisplayFilter| {
        let url = format!("/home?meta_id={}", filter.base_model_id);
        navigate(&url, Default::default());
    };

    view! {
        <div class="min-h-screen bg-base-100 pb-20 pt-8">
            <div class="container mx-auto px-4 max-w-6xl">

                // --- 1. 顶部标题 & 搜索 ---
                <section class="text-center mb-12">
                    <h2 class="text-3xl font-bold mb-4 text-dark flex items-center justify-center">
                        <i class="fa-solid fa-wand-magic-sparkles text-secondary mr-3"></i>
                        "声音滤镜库"
                    </h2>
                    <p class="text-gray-500 mb-8">
                        "发现并使用各种声音滤镜，一键应用到你的作品中"
                    </p>

                    <div class="relative max-w-xl mx-auto">
                        <input
                            type="text"
                            placeholder="搜索滤镜（名称、作者、标签、描述）"
                            class="w-full pl-12 pr-10 py-3 rounded-full border border-gray-200 bg-white focus:outline-none focus:ring-2 focus:ring-primary/50 shadow-sm"
                            prop:value=move || search.get()
                            on:input=move |ev| set_search.set(event_target_value(&ev))
                        />
                        <i class="fa-solid fa-magnifying-glass absolute left-5 top-1/2 transform -translate-y-1/2 text-gray-400"></i>
                        <Show when=move || !search.get().is_empty()>
                            <button
                                class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 px-2"
                                on:click=move |_| set_search.set(String::new())
                                title="清除"
                            >
                                <i class="fa-solid fa-xmark"></i>
                            </button>
                        </Show>
                    </div>
                </section>

                <Suspense fallback=move || {
                    view! { <div class="text-center py-10">"加载滤镜中..."</div> }
                }>
                    {move || {
                        let recommended_res = recommended_filters_resource.get();
                        let normal_res = normal_filters_resource.get();
                        match (recommended_res, normal_res) {
                            (Some(Ok(recommended_meta_list)), Some(Ok(normal_meta_list))) => {
                                let mut all_filters: Vec<DisplayFilter> = recommended_meta_list
                                    .into_iter()
                                    .map(DisplayFilter::from_voice_meta)
                                    .collect();
                                let normal_filters: Vec<DisplayFilter> = normal_meta_list
                                    .into_iter()
                                    .map(DisplayFilter::from_voice_meta)
                                    .collect();
                                all_filters.extend(normal_filters);
                                let query = search.get().trim().to_lowercase();
                                let filtered: Vec<DisplayFilter> = if query.is_empty() {
                                    all_filters.clone()
                                } else {
                                    all_filters
                                        .into_iter()
                                        .filter(|f| {
                                            let name = f.name.to_lowercase();
                                            let author = f.author.to_lowercase();
                                            let desc = f.description.to_lowercase();
                                            name.contains(&query) || author.contains(&query)
                                                || desc.contains(&query)
                                        })
                                        .collect()
                                };
                                let (official, user): (Vec<_>, Vec<_>) = filtered
                                    .into_iter()
                                    .partition(|f| f.is_official);
                                let apply_filter_1 = apply_filter.clone();
                                let apply_filter_2 = apply_filter.clone();

                                view! {
                                    <div class="space-y-16">
                                        <FilterSection
                                            title="官方推荐滤镜"
                                            icon="fa-star"
                                            icon_color="text-primary"
                                            filters=official
                                            on_apply=apply_filter_1
                                        />

                                        <FilterSection
                                            title="用户分享滤镜"
                                            icon="fa-users"
                                            icon_color="text-secondary"
                                            filters=user
                                            on_apply=apply_filter_2
                                        />
                                    </div>
                                }
                                    .into_any()
                            }
                            (Some(Err(e)), _) | (_, Some(Err(e))) => {
                                view! {
                                    <div class="text-red-500 text-center">
                                        "加载失败: " {e.to_string()}
                                    </div>
                                }
                                    .into_any()
                            }
                            _ => view! { <div class="text-center">"加载中..."</div> }.into_any(),
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}

// --- 子组件：滤镜区域 ---
#[component]
fn FilterSection<F>(
    title: &'static str,
    icon: &'static str,
    icon_color: &'static str,
    filters: Vec<DisplayFilter>,
    on_apply: F, // 泛型闭包
) -> impl IntoView
where
    F: Fn(DisplayFilter) + Clone + Send + 'static,
{
    view! {
        <section>
            <h3 class="text-xl font-bold mb-6 flex items-center text-gray-800">
                <i class=format!("fa {} {} mr-2", icon, icon_color)></i>
                {title}
            </h3>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                <For
                    each=move || filters.clone()
                    key=|f| f.id.clone()
                    children=move |filter| {
                        let filter_clone = filter.clone();
                        let on_apply = on_apply.clone();
                        let filter_name = StoredValue::new(filter.name.clone());
                        let filter_description = filter.description.clone();
                        let filter_author = filter.author.clone();
                        let filter_author_id = filter.author_id;
                        let filter_is_official = filter.is_official;
                        let (is_liked, set_is_liked) = signal(filter.is_liked);
                        let (likes_count, set_likes_count) = signal(filter.likes_count);
                        let (show_comments, set_show_comments) = signal(false);
                        let (comments, set_comments) = signal(Vec::<(String, String)>::new());
                        let (new_comment, set_new_comment) = signal(String::new());
                        let (comments_count, set_comments_count) = signal(filter.comments_count);
                        let post_id = filter.id.clone();
                        let post_id_like = StoredValue::new(post_id.clone());
                        let post_id_comment = StoredValue::new(post_id.clone());
                        let post_id_fetch = StoredValue::new(post_id.clone());

                        view! {
                            <div class="bg-white rounded-xl p-6 shadow-soft hover:shadow-lg transition-all duration-300 border border-gray-100 group flex flex-col">
                                <div class="flex justify-between items-start mb-3">
                                    <h4 class="text-lg font-bold text-gray-800">
                                        {filter_name.get_value()}
                                    </h4>
                                    {if filter_is_official {
                                        view! {
                                            <span class="text-xs px-2 py-1 rounded-full bg-yellow-100 text-yellow-600">
                                                "官方"
                                            </span>
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <div class="w-8 h-8 rounded-full overflow-hidden bg-gray-200 flex items-center justify-center">
                                                <img
                                                    src=format!("/api/avatar/{}", filter_author_id)
                                                    alt=filter_author.clone()
                                                    class="w-full h-full object-cover"
                                                />
                                            </div>
                                        }
                                            .into_any()
                                    }}
                                </div>

                                <div class="h-28 overflow-y-auto mb-4 text-sm text-gray-500 whitespace-pre-wrap break-words">
                                    {filter_description.clone()}
                                </div>

                                <div class="flex items-center justify-between mt-auto pt-4">
                                    <div class="text-xs text-gray-400 flex items-center space-x-3">
                                        <button
                                            class=move || {
                                                format!(
                                                    "flex items-center hover:text-primary cursor-pointer transition-colors {}",
                                                    if is_liked.get() { "text-red-500" } else { "" },
                                                )
                                            }
                                            on:click=move |_| {
                                                let post_id = post_id_like.get_value();
                                                spawn_local(async move {
                                                    if let Ok(_) = toggle_meta_post_like(post_id).await {
                                                        let new_liked = !is_liked.get();
                                                        set_is_liked.set(new_liked);
                                                        if new_liked {
                                                            set_likes_count.set(likes_count.get() + 1);
                                                        } else {
                                                            set_likes_count.set(likes_count.get() - 1);
                                                        }
                                                    }
                                                });
                                            }
                                        >
                                            <i class=move || {
                                                if is_liked.get() {
                                                    "fa-solid fa-heart mr-1"
                                                } else {
                                                    "fa-regular fa-heart mr-1"
                                                }
                                            }></i>
                                            {move || likes_count.get()}
                                        </button>
                                        <button
                                            class="flex items-center hover:text-primary cursor-pointer transition-colors"
                                            on:click=move |_| {
                                                let should_show = !show_comments.get();
                                                set_show_comments.set(should_show);
                                                if should_show && comments.get().is_empty() {
                                                    let post_id = post_id_fetch.get_value();
                                                    spawn_local(async move {
                                                        if let Ok(fetched_comments) = get_meta_post_comments(
                                                                post_id,
                                                            )
                                                            .await
                                                        {
                                                            set_comments.set(fetched_comments);
                                                        }
                                                    });
                                                }
                                            }
                                        >
                                            <i class="fa-regular fa-comment mr-1"></i>
                                            {move || comments_count.get()}
                                        </button>
                                    </div>

                                    <button
                                        class="bg-primary/10 hover:bg-primary text-primary hover:text-white px-4 py-2 rounded-lg text-sm font-medium transition-colors flex items-center"
                                        on:click=move |_| {
                                            on_apply(filter_clone.clone());
                                        }
                                    >
                                        <i class="fa-solid fa-check mr-2"></i>
                                        "使用"
                                    </button>
                                </div>

                                // 评论模态框
                                <Show when=move || {
                                    show_comments.get()
                                }>
                                    {move || {
                                        let post_id_comment_val = post_id_comment.get_value();
                                        let name = filter_name.get_value();

                                        view! {
                                            <div
                                                class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4"
                                                on:click=move |_| set_show_comments.set(false)
                                            >
                                                <div
                                                    class="bg-white rounded-lg max-w-2xl w-full max-h-[80vh] flex flex-col"
                                                    on:click=|e| e.stop_propagation()
                                                >
                                                    <div class="flex justify-between items-center p-4 border-b">
                                                        <h3 class="text-lg font-bold text-gray-800">
                                                            {name}"的评论"
                                                        </h3>
                                                        <button
                                                            class="text-gray-400 hover:text-gray-600 transition-colors"
                                                            on:click=move |_| set_show_comments.set(false)
                                                        >
                                                            <i class="fa-solid fa-xmark text-xl"></i>
                                                        </button>
                                                    </div>

                                                    <div class="flex-1 overflow-y-auto p-4 space-y-3">
                                                        <For
                                                            each=move || comments.get()
                                                            key=|(user_id, _)| user_id.clone()
                                                            children=move |(user_id, content)| {
                                                                view! {
                                                                    <div class="flex items-start gap-3 p-3 bg-gray-50 rounded-lg">
                                                                        <img
                                                                            src=format!("/api/avatar/{}", user_id)
                                                                            alt="用户头像"
                                                                            class="w-8 h-8 rounded-full object-cover flex-shrink-0"
                                                                        />
                                                                        <div class="flex-1 min-w-0">
                                                                            <p class="text-sm text-gray-800 break-words">{content}</p>
                                                                        </div>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                        <Show when=move || comments.get().is_empty()>
                                                            <div class="text-center py-8 text-gray-400">
                                                                "还没有评论，快来抢沙发吧~"
                                                            </div>
                                                        </Show>
                                                    </div>

                                                    <div class="p-4 border-t">
                                                        <div class="flex items-center gap-2">
                                                            <input
                                                                type="text"
                                                                placeholder="添加评论..."
                                                                class="flex-1 px-4 py-2 border border-gray-200 rounded-full focus:outline-none focus:ring-2 focus:ring-primary/50"
                                                                prop:value=move || new_comment.get()
                                                                on:input=move |ev| {
                                                                    set_new_comment.set(event_target_value(&ev))
                                                                }
                                                            />
                                                            <button
                                                                class="bg-primary text-white px-6 py-2 rounded-full hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                                                                disabled=move || new_comment.get().trim().is_empty()
                                                                on:click={
                                                                    let post_id_for_click = post_id_comment_val.clone();
                                                                    move |_| {
                                                                        let comment = new_comment.get();
                                                                        if !comment.trim().is_empty() {
                                                                            let post_id = post_id_for_click.clone();
                                                                            let comment_clone = comment.clone();
                                                                            spawn_local(async move {
                                                                                if let Ok(_) = add_meta_post_comment(
                                                                                        post_id.clone(),
                                                                                        comment_clone.clone(),
                                                                                    )
                                                                                    .await
                                                                                {
                                                                                    set_new_comment.set(String::new());
                                                                                    set_comments_count.set(comments_count.get() + 1);
                                                                                    if let Ok(fetched_comments) = get_meta_post_comments(
                                                                                            post_id,
                                                                                        )
                                                                                        .await
                                                                                    {
                                                                                        set_comments.set(fetched_comments);
                                                                                    }
                                                                                }
                                                                            });
                                                                        }
                                                                    }
                                                                }
                                                            >
                                                                "发送"
                                                            </button>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }}
                                </Show>
                            </div>
                        }
                    }
                />
            </div>
        </section>
    }
}
