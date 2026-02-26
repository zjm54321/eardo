use crate::api::post::{PostStatus, VoicePost, list_voice_post};
use crate::pages::component::CustomAudioPlayer;
#[cfg(feature = "ssr")]
use crate::api::post::{Action, VoicePostID, action_voice_post, get_voice_post_comments};
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use std::str::FromStr;

#[server]
pub async fn toggle_post_like(post_id: String) -> Result<(), ServerFnError> {
    let id = VoicePostID::from_str(&post_id).map_err(|e| ServerFnError::new(e.to_string()))?;
    action_voice_post(id, Action::LikeDislike).await
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentInfo {
    pub id: String,
    pub author: String,
    pub avatar: Option<String>,
    pub content: String,
    pub time: String,
}

#[server]
pub async fn get_post_comments(post_id: String) -> Result<Vec<CommentInfo>, ServerFnError> {
    let id = VoicePostID::from_str(&post_id).map_err(|e| ServerFnError::new(e.to_string()))?;
    let comments = get_voice_post_comments(id).await?;
    Ok(comments
        .into_iter()
        .map(|(user_id, content)| CommentInfo {
            id: "".to_string(), // TODO: 评论ID
            author: user_id.to_string(),
            avatar: Some(format!("/api/avatar/{}", user_id)),
            content,
            time: "".to_string(), // TODO: 评论时间
        })
        .collect())
}

#[server]
pub async fn add_post_comment(post_id: String, content: String) -> Result<(), ServerFnError> {
    let id = VoicePostID::from_str(&post_id).map_err(|e| ServerFnError::new(e.to_string()))?;
    action_voice_post(id, Action::Comment(content)).await
}

// 用于在页面中使用的扁平化的 PostMetadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostMetadata {
    pub author: String,
    pub avatar: String,
    pub time: String,
    pub description: String,
    pub likes: i32,
    pub comments: i32,
    pub voice_type: String,
    pub audio_url: String,
}

impl PostMetadata {
    pub fn from_post(post: &VoicePost) -> Self {
        Self {
            author: post.author.to_string(),
            avatar: format!("/api/avatar/{}", post.author),
            time: "".to_string(), // TODO: 时间
            description: post.content.clone(),
            likes: post.likes_count,
            comments: post.comments_count,
            voice_type: "".to_string(), // TODO: 声音类型
            audio_url: format!("/api/audio/{}", post.library_id),
        }
    }
}


#[component]
pub fn Playground() -> impl IntoView {
    // --- 状态管理 ---
    // 1. 轮播图状态
    let (current_slide, set_current_slide) = signal(0);

    // 2. 列表作品状态
    let (works_list, set_works_list) = signal(Vec::<VoicePost>::new());
    let (featured_list, set_featured_list) = signal(Vec::<VoicePost>::new());
    let (search_query, set_search_query) = signal(String::new());

    // 加载精选作品 (Recommended状态)
    let featured_resource = LocalResource::new(move || async move {
        match list_voice_post(Some(PostStatus::Recommended), 20).await {
            Ok(posts) => Ok(posts),
            Err(e) => {
                leptos::logging::error!("加载精选作品失败: {}", e);
                Err(e)
            }
        }
    });

    // 加载最新作品 (Normal状态)
    let posts_resource = LocalResource::new(move || async move {
        match list_voice_post(Some(PostStatus::Normal), 20).await {
            Ok(posts) => Ok(posts),
            Err(e) => {
                leptos::logging::error!("加载作品失败: {}", e);
                Err(e)
            }
        }
    });

    Effect::new(move || {
        if let Some(Ok(featured)) = featured_resource.get() {
            set_featured_list.set(featured);
        }
        if let Some(Ok(posts)) = posts_resource.get() {
            set_works_list.set(posts);
        }
    });

    // --- 交互逻辑 ---
    let next_slide = move |total: usize| {
        set_current_slide.update(|i| *i = (*i + 1) % total);
    };

    let prev_slide = move |total: usize| {
        set_current_slide.update(|i| *i = (*i + total - 1) % total);
    };

    view! {
        <div class="min-h-screen bg-base-100 pb-20 pt-8">
            <div class="container mx-auto px-4 max-w-6xl">

                // --- 1. 顶部标题 ---
                <section class="text-center mb-12">
                    <h2 class="text-[clamp(1.8rem,4vw,2.5rem)] font-bold mb-4 text-shadow flex items-center justify-center">
                        <i class="fa-solid fa-users text-secondary mr-2"></i>
                        "声音广场"
                    </h2>
                    <p class="text-gray-600 max-w-2xl mx-auto">
                        "分享您的创意声音作品，聆听他人的声音世界"
                    </p>
                </section>

                // --- 2. 精选作品轮播 ---
                <section class="mb-16">
                    <Suspense fallback=|| {
                        view! { <div class="text-center py-10">"加载精选作品..."</div> }
                    }>
                        {move || {
                            let featured = featured_list.get();
                            if !featured.is_empty() {
                                let total = featured.len();
                                view! {
                                    <div class="relative">
                                        // 标题栏 + 控制按钮
                                        <div class="flex justify-between items-center mb-6">
                                            <h3 class="text-xl font-semibold">"精选作品"</h3>
                                            <div class="flex space-x-2">
                                                <button
                                                    class="w-10 h-10 rounded-full border border-gray-300 flex items-center justify-center hover:border-primary hover:text-primary transition-all duration-300"
                                                    on:click=move |_| prev_slide(total)
                                                >
                                                    <i class="fa-solid fa-chevron-left"></i>
                                                </button>
                                                <button
                                                    class="w-10 h-10 rounded-full border border-gray-300 flex items-center justify-center hover:border-primary hover:text-primary transition-all duration-300"
                                                    on:click=move |_| next_slide(total)
                                                >
                                                    <i class="fa-solid fa-chevron-right"></i>
                                                </button>
                                            </div>
                                        </div>

                                        // 轮播内容
                                        <div class="relative overflow-hidden rounded-2xl min-h-[300px] px-8 md:px-16 lg:px-24">
                                            {featured
                                                .into_iter()
                                                .enumerate()
                                                .map(|(idx, work)| {
                                                    view! {
                                                        <div
                                                            class="transition-all duration-500 ease-in-out"
                                                            class:hidden=move || current_slide.get() != idx
                                                            class:block=move || current_slide.get() == idx
                                                            class:animate-fade-in=move || current_slide.get() == idx
                                                        >
                                                            <WorkCard work=work is_featured=true />
                                                        </div>
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>

                                        // 指示器
                                        <div class="flex justify-center mt-6 space-x-2">
                                            {(0..total)
                                                .map(|idx| {
                                                    view! {
                                                        <button
                                                            class="w-3 h-3 rounded-full transition-colors duration-300"
                                                            class:bg-primary=move || current_slide.get() == idx
                                                            class:bg-gray-300=move || current_slide.get() != idx
                                                            on:click=move |_| set_current_slide.set(idx)
                                                        />
                                                    }
                                                })
                                                .collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="text-gray-500 text-center">
                                        "暂无精选作品"
                                    </div>
                                }
                                    .into_any()
                            }
                        }}
                    </Suspense>
                </section>

                // --- 3. 最新作品列表 + 搜索 ---
                <section>
                    <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-3 mb-6">
                        <h3 class="text-xl font-semibold">"最新作品"</h3>
                        <div class="flex items-center gap-2 w-full md:w-auto">
                            <div class="flex-1 md:flex-initial md:w-72 lg:w-80">
                                <input
                                    class="w-full px-3 py-2 border border-gray-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary/50 text-sm"
                                    type="text"
                                    placeholder="搜索标题、作者或描述"
                                    prop:value=move || search_query.get()
                                    on:input=move |ev| set_search_query.set(event_target_value(&ev))
                                />
                            </div>
                            <button
                                class="text-sm text-gray-500 hover:text-primary px-3 py-2"
                                on:click=move |_| set_search_query.set(String::new())
                            >
                                "清除"
                            </button>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                        <For
                            each=move || {
                                let query = search_query.get().trim().to_lowercase();
                                let posts = works_list.get();
                                if query.is_empty() {
                                    posts
                                } else {
                                    posts
                                        .into_iter()
                                        .filter(|p| {
                                            let meta = PostMetadata::from_post(p);
                                            p.title.to_lowercase().contains(&query)
                                                || meta.description.to_lowercase().contains(&query)
                                                || meta.author.to_lowercase().contains(&query)
                                        })
                                        .collect()
                                }
                            }
                            key=|w| w.id.to_string()
                            children=move |work| {
                                view! { <WorkCard work=work is_featured=false /> }
                            }
                        />
                    </div>

                    // 暂无更多作品提示
                    {move || {
                        if works_list.get().is_empty() {
                            view! {
                                <div class="text-center mt-10 text-gray-500">
                                    "暂无更多作品"
                                </div>
                            }
                                .into_any()
                        } else {
                            view! { <div></div> }.into_any()
                        }
                    }}
                </section>

            </div>
        </div>
    }
}

// --- 子组件：作品卡片 ---
#[component]
fn WorkCard(work: VoicePost, is_featured: bool) -> impl IntoView {
    let meta = PostMetadata::from_post(&work);
    let post_id = work.id.to_string();
    let post_id_for_comment = work.id.to_string();

    // 从 work.meta 中获取 is_liked 状态
    let initial_liked = false; // TODO: is_liked

    let (liked, set_liked) = signal(initial_liked);
    let (like_count, set_like_count) = signal(meta.likes);
    let (is_loading, set_is_loading) = signal(false);

    // 评论相关状态
    let (show_comments, set_show_comments) = signal(false);
    let (comment_count, set_comment_count) = signal(meta.comments);

    let toggle_like = move |_| {
        if is_loading.get() {
            return;
        }

        let post_id = post_id.clone();
        let was_liked = liked.get();

        // 乐观更新UI
        set_liked.update(|v| *v = !*v);
        set_like_count.update(|c| if was_liked { *c -= 1 } else { *c += 1 });
        set_is_loading.set(true);

        spawn_local(async move {
            match toggle_post_like(post_id).await {
                Ok(_) => {
                    leptos::logging::debug_log!("点赞操作成功");
                }
                Err(e) => {
                    leptos::logging::error!("点赞失败: {}", e);
                    // 恢复原状态
                    set_liked.set(was_liked);
                    set_like_count.update(|c| if was_liked { *c += 1 } else { *c -= 1 });
                }
            }
            set_is_loading.set(false);
        });
    };

    view! {
        <div class="relative">
            <div class="bg-white rounded-xl p-6 shadow-soft transition-all duration-300 hover:shadow-hover min-h-[400px] flex flex-col">
                // 头像（左侧）和标题（右侧）
                <div class="flex items-start mb-4">
                    // 用户头像（左侧）
                    <div class="w-12 h-12 rounded-full overflow-hidden bg-gray-200 flex items-center justify-center mr-3 flex-shrink-0">
                        <img src=meta.avatar alt="Avatar" class="w-full h-full object-cover" />
                    </div>
                    <div class="flex-1 min-w-0">
                        // 标题
                        <h3 class="font-semibold text-lg text-dark">{work.title.clone()}</h3>
                    </div>
                </div>

                // 描述内容（可滚动）
                <div
                    class="text-sm text-gray-600 mb-4 overflow-y-auto flex-grow"
                    class:max-h-32=move || !is_featured
                    class:max-h-40=is_featured
                >
                    {meta.description}
                </div>

                // 音频播放器（自定义样式）
                <CustomAudioPlayer audio_url=meta.audio_url.clone() />

                // 底部操作栏 (mt-auto 保证对齐底部)
                <div class="flex justify-between items-center text-sm mt-auto pt-2">
                    <div class="flex items-center text-gray-500 space-x-4">
                        <button
                            class="flex items-center transition-colors duration-200 group"
                            class:text-red-500=move || liked.get()
                            class:text-gray-500=move || !liked.get()
                            on:click=toggle_like
                            disabled=move || is_loading.get()
                        >
                            {move || {
                                if liked.get() {
                                    view! { <i class="fa-solid fa-heart mr-1"></i> }
                                } else {
                                    view! { <i class="fa-regular fa-heart mr-1"></i> }
                                }
                            }}
                            <span>{move || like_count.get()}</span>
                        </button>
                        <button
                            class="flex items-center hover:text-primary transition-colors duration-200"
                            on:click=move |_| set_show_comments.update(|v| *v = !*v)
                        >
                            <i class="fa-solid fa-comment mr-1"></i>
                            <span>{move || comment_count.get()}</span>
                        </button>
                    </div>
                    <span class="text-secondary bg-secondary/10 px-2 py-1 rounded-full text-xs">
                        {meta.voice_type}
                    </span>
                </div>

            </div>

            // 评论弹出窗口
            {move || {
                if show_comments.get() {
                    view! {
                        <div
                            class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4"
                            on:click=move |_| set_show_comments.set(false)
                        >
                            <div
                                class="bg-white rounded-xl shadow-2xl w-full max-w-2xl max-h-[80vh] flex flex-col"
                                on:click=move |e| e.stop_propagation()
                            >
                                <div class="flex justify-between items-center p-4 border-b border-gray-200">
                                    <h3 class="text-lg font-semibold">"评论"</h3>
                                    <button
                                        class="text-gray-400 hover:text-gray-600 transition-colors text-2xl leading-none px-2"
                                        on:click=move |_| set_show_comments.set(false)
                                    >
                                        "×"
                                    </button>
                                </div>
                                <div class="flex-1 overflow-y-auto p-4">
                                    <CommentSection
                                        post_id=post_id_for_comment.clone()
                                        on_comment_added=move |_| {
                                            set_comment_count.update(|c| *c += 1)
                                        }
                                    />
                                </div>
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}
        </div>
    }
}

// 评论区组件
#[component]
fn CommentSection<F>(post_id: String, on_comment_added: F) -> impl IntoView
where
    F: Fn(()) + 'static + Clone,
{
    let post_id_clone = post_id.clone();
    let (comment_text, set_comment_text) = signal(String::new());
    let (is_submitting, set_is_submitting) = signal(false);

    let comments_resource = LocalResource::new(move || {
        let pid = post_id.clone();
        async move { get_post_comments(pid).await }
    });

    let submit_comment = move |_| {
        if comment_text.get().trim().is_empty() || is_submitting.get() {
            return;
        }

        let content = comment_text.get();
        let pid = post_id_clone.clone();
        let on_added = on_comment_added.clone();
        set_is_submitting.set(true);

        spawn_local(async move {
            match add_post_comment(pid, content).await {
                Ok(_) => {
                    leptos::logging::debug_log!("评论发表成功");
                    set_comment_text.set(String::new());
                    on_added(());
                    // 刷新评论列表
                    comments_resource.refetch();
                }
                Err(e) => {
                    leptos::logging::error!("评论失败: {}", e);
                }
            }
            set_is_submitting.set(false);
        });
    };

    view! {
        <div>
            // 发表评论
            <div class="mb-4">
                <textarea
                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary resize-none"
                    rows="3"
                    placeholder="写下你的评论..."
                    prop:value=move || comment_text.get()
                    on:input=move |e| set_comment_text.set(event_target_value(&e))
                />
                <div class="flex justify-end mt-2">
                    <button
                        class="px-4 py-1.5 bg-primary text-white rounded-lg hover:bg-primary/90 transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm"
                        on:click=submit_comment
                        disabled=move || is_submitting.get() || comment_text.get().trim().is_empty()
                    >
                        {move || if is_submitting.get() { "发表中..." } else { "发表评论" }}
                    </button>
                </div>
            </div>

            // 评论列表
            <div class="border-t border-gray-200 pt-4">
                <h4 class="text-sm font-semibold text-gray-700 mb-3">"全部评论"</h4>
                <div class="space-y-3 max-h-96 overflow-y-auto">
                    <Suspense fallback=|| {
                        view! {
                            <div class="text-center text-gray-500 py-2">"加载评论..."</div>
                        }
                    }>
                        {move || {
                            comments_resource
                                .get()
                                .map(|result| {
                                    match result {
                                        Ok(comments) if !comments.is_empty() => {
                                            view! {
                                                <div class="space-y-3">
                                                    <For
                                                        each=move || comments.clone()
                                                        key=|c| c.id.clone()
                                                        children=move |comment| {
                                                            view! {
                                                                <div class="flex items-start space-x-2">
                                                                    <img
                                                                        src=comment
                                                                            .avatar
                                                                            .unwrap_or_else(|| {
                                                                                "https://apiold.dicebear.com/7.x/avataaars/svg?seed=comment"
                                                                                    .to_string()
                                                                            })
                                                                        class="w-8 h-8 rounded-full object-cover flex-shrink-0"
                                                                    />
                                                                    <div class="flex-1 min-w-0">
                                                                        <div class="flex items-center space-x-2">
                                                                            <span class="font-medium text-sm text-gray-800">
                                                                                {comment.author}
                                                                            </span>
                                                                            <span class="text-xs text-gray-400">{comment.time}</span>
                                                                        </div>
                                                                        <p class="text-sm text-gray-600 mt-1">{comment.content}</p>
                                                                    </div>
                                                                </div>
                                                            }
                                                        }
                                                    />
                                                </div>
                                            }
                                                .into_any()
                                        }
                                        Ok(_) => {
                                            view! {
                                                <div class="text-center text-gray-400 py-4 text-sm">
                                                    "暂无评论，快来抢沙发吧~"
                                                </div>
                                            }
                                                .into_any()
                                        }
                                        Err(e) => {
                                            view! {
                                                <div class="text-center text-red-500 py-2 text-sm">
                                                    {format!("加载失败: {}", e)}
                                                </div>
                                            }
                                                .into_any()
                                        }
                                    }
                                })
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
