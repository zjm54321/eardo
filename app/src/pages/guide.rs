use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[component]
pub fn GuideButton() -> impl IntoView {
    let is_open = RwSignal::new(false);
    let (is_visible, set_is_visible) = signal(true);
    
    // 存储当前激活的浮层元素ID
    let active_popup = RwSignal::new(None::<String>);

    // 移除所有浮层
    let remove_all_popups = move || {
        if let Some(current_id) = active_popup.get_untracked() {
            if let Some(popup) = document().get_element_by_id(&current_id) {
                let _ = popup.remove();
            }
            active_popup.set(None);
        }
    };

    // 创建浮层
    let create_popup = move |target_id: &str, title: &str, video_url: &str, description: &str| {
        let target_id = target_id.to_string();
        let title = title.to_string();
        let video_url = video_url.to_string();
        let description = description.to_string();
        
        // 先移除已有的浮层
        remove_all_popups();
        
        // 获取目标元素的位置
        if let Some(target) = document().get_element_by_id(&target_id) {
            // 获取元素当前相对于视口的位置（fixed 定位直接用这个坐标）
            let rect = target.get_bounding_client_rect();
            
            // 创建浮层容器
            let popup_id = format!("popup-{}", target_id);
            let popup_div = document().create_element("div").unwrap();
            popup_div.set_id(&popup_id);
            popup_div.set_attribute("class", "fixed z-50 bg-white rounded-xl shadow-2xl border border-gray-200 w-80 animate-fade-in").unwrap();
            
            // 计算水平位置（使用视口坐标）
            let popup_width = 320.0;
            let spacing = 10.0;
            let viewport_width = window().inner_width().unwrap().as_f64().unwrap_or(0.0);
            
            let left_pos = if rect.right() + popup_width + spacing < viewport_width {
                // 右侧显示：弹窗左边缘 = 元素右边缘 + 间距
                rect.right() + spacing
            } else {
                // 左侧显示：弹窗右边缘 = 元素左边缘 - 间距
                rect.left() - popup_width - spacing
            };
            
            // 计算垂直位置（fixed 定位，直接使用视口坐标，不加 scroll_y）
            let top_pos = rect.top();  // 顶部对齐
            
            // 调试输出
            web_sys::console::log_1(&format!(
                "创建弹窗 - 目标元素位置: top={}, left={}, right={}, bottom={}",
                rect.top(), rect.left(), rect.right(), rect.bottom()
            ).into());
            web_sys::console::log_1(&format!(
                "弹窗位置: top={}, left={}, viewport_width={}",
                top_pos, left_pos, viewport_width
            ).into());
            
            popup_div.set_attribute(
                "style",
                &format!(
                    "top: {}px; left: {}px;",
                    top_pos,
                    left_pos
                ),
            ).unwrap();
            
            // 浮层内容
            popup_div.set_inner_html(&format!(
                r#"
                <div class="relative">
                    <button class="absolute top-2 right-2 w-6 h-6 rounded-full bg-gray-100 hover:bg-gray-200 flex items-center justify-center text-gray-500 transition-colors z-10" id="close-popup-{}">
                        <i class="fa-solid fa-times text-xs"></i>
                    </button>
                    <div class="p-4">
                        <h4 class="font-semibold text-gray-800 mb-2 flex items-center gap-2">
                            <i class="fa-solid fa-play-circle text-primary"></i>
                            {}
                        </h4>
                        <div class="mb-3 rounded-lg overflow-hidden bg-gray-100 aspect-video">
                            <video 
                                src="{}" 
                                controls 
                                class="w-full h-full object-cover"
                                poster=""
                            >
                                您的浏览器不支持视频播放
                            </video>
                        </div>
                        <p class="text-sm text-gray-600 leading-relaxed">
                            {}
                        </p>
                    </div>
                </div>
                "#,
                popup_id, title, video_url, description
            ));
            
            // 添加到body
            document().body().unwrap().append_child(&popup_div).unwrap();
            
            // 绑定关闭按钮事件
            let close_btn = document().get_element_by_id(&format!("close-popup-{}", popup_id)).unwrap();
            let popup_id_clone = popup_id.clone();
            let popup_id_clone2 = popup_id.clone();
            let active_popup_clone = active_popup;
            let active_popup_clone2 = active_popup;
            let close_handler = Closure::wrap(Box::new(move || {
                if let Some(popup) = document().get_element_by_id(&popup_id_clone) {
                    let _ = popup.remove();
                }
                active_popup_clone.set(None);
            }) as Box<dyn FnMut()>);
            
            close_btn.add_event_listener_with_callback("click", close_handler.as_ref().unchecked_ref()).unwrap();
            close_handler.forget();
            
            active_popup.set(Some(popup_id));
            
            // 点击外部关闭
            let doc = document();
            let click_outside_handler = Closure::wrap(Box::new(move |e: web_sys::Event| {
                let target = e.target().unwrap();
                let popup_exists = document().get_element_by_id(&popup_id_clone2).is_some();
                if popup_exists {
                    if let Some(popup) = document().get_element_by_id(&popup_id_clone2) {
                        let target_node: Option<&web_sys::Node> = target.dyn_ref();
                        if let Some(node) = target_node {
                            if !popup.contains(Some(node)) {
                                let _ = popup.remove();
                                active_popup_clone2.set(None);
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(web_sys::Event)>);
            doc.add_event_listener_with_callback("click", click_outside_handler.as_ref().unchecked_ref()).unwrap();
            click_outside_handler.forget();
        }
    };

    // 滚动元素到屏幕顶部的辅助函数（用于声线选择，避开导航栏）
    let scroll_element_to_safe_top = move |element: &web_sys::Element| {
        let rect = element.get_bounding_client_rect();
        let window = window();
        let scroll_y = window.scroll_y().unwrap_or(0.0);
        
        // 计算目标滚动位置：让元素顶部距离视口顶部120px（避开导航栏，导航栏通常高度60-80px）
        let target_scroll_y = rect.top() + scroll_y - 120.0;
        
        // 平滑滚动到目标位置
        let _ = window.scroll_to_with_x_and_y(0.0, target_scroll_y.max(0.0));
    };

    // 滚动元素到屏幕中央的辅助函数（用于其他部分）
    let scroll_element_to_center = move |element: &web_sys::Element| {
        let rect = element.get_bounding_client_rect();
        let window = window();
        let scroll_y = window.scroll_y().unwrap_or(0.0);
        let viewport_height = window.inner_height().unwrap().as_f64().unwrap_or(0.0);
        
        // 计算元素中心点相对于文档顶部的距离
        let element_center = rect.top() + rect.height() / 2.0 + scroll_y;
        // 计算目标滚动位置：让元素中心在视口中心
        let target_scroll_y = element_center - viewport_height / 2.0;
        
        // 平滑滚动到目标位置
        let _ = window.scroll_to_with_x_and_y(0.0, target_scroll_y.max(0.0));
    };

    // 等待滚动完成后创建弹窗的辅助函数
    let create_popup_after_scroll = move |id: String, title: String, video_url: String, description: String| {
        let window = window();
        let create_popup_clone = create_popup.clone();
        
        // 在第一帧之前就 clone window
        let window_clone1 = window.clone();
        let closure1 = Closure::wrap(Box::new(move || {
            let window_clone2 = window_clone1.clone();
            let create_popup_clone2 = create_popup_clone.clone();
            let id2 = id.clone();
            let title2 = title.clone();
            let video_url2 = video_url.clone();
            let description2 = description.clone();
            
            // 第二帧
            let closure2 = Closure::wrap(Box::new(move || {
                create_popup_clone2(&id2, &title2, &video_url2, &description2);
            }) as Box<dyn FnMut()>);
            
            let _ = window_clone2.request_animation_frame(closure2.as_ref().unchecked_ref());
            closure2.forget();
        }) as Box<dyn FnMut()>);
        
        let _ = window.request_animation_frame(closure1.as_ref().unchecked_ref());
        closure1.forget();
    };

    // 滚动到指定区域并显示浮层
    let scroll_to_element = move |id: &str, title: &str, video_url: &str, description: &str, is_voice_selector: bool| {
        let id = id.to_string();
        let title = title.to_string();
        let video_url = video_url.to_string();
        let description = description.to_string();
        
        set_is_visible.set(false);
        
        // 使用微小的延迟让界面先响应
        set_timeout(
            move || {
                let doc = web_sys::window().unwrap().document().unwrap();
                if let Some(element) = doc.get_element_by_id(&id) {
                    // 根据元素类型选择滚动方式
                    if is_voice_selector {
                        // 声线选择：滚动到安全位置（避开导航栏）
                        scroll_element_to_safe_top(&element);
                    } else {
                        // 其他部分：滚动到屏幕中央
                        scroll_element_to_center(&element);
                    }
                    
                    // 高亮动画
                    let class_list = element.class_list();
                    let _ = class_list.add_1("ring-2");
                    let _ = class_list.add_1("ring-primary");
                    let _ = class_list.add_1("ring-offset-2");
                    
                    let id_clone = id.clone();
                    let title_clone = title.clone();
                    let video_clone = video_url.clone();
                    let desc_clone = description.clone();
                    
                    // 等待滚动完成后创建弹窗
                    let create_after_scroll = create_popup_after_scroll.clone();
                    create_after_scroll(id_clone, title_clone, video_clone, desc_clone);
                    
                    // 移除高亮动画
                    let id_clone2 = id.clone();
                    set_timeout(
                        move || {
                            let doc = web_sys::window().unwrap().document().unwrap();
                            if let Some(el) = doc.get_element_by_id(&id_clone2) {
                                let _ = el.class_list().remove_1("ring-2");
                                let _ = el.class_list().remove_1("ring-primary");
                                let _ = el.class_list().remove_1("ring-offset-2");
                            }
                        },
                        std::time::Duration::from_millis(1500),
                    );
                }
                set_is_visible.set(true);
                is_open.set(false);
            },
            std::time::Duration::from_millis(50),
        );
    };

    view! {
        <Show when=move || is_visible.get()>
            <div class="fixed bottom-3 md:bottom-6 right-3 md:right-6 z-50">
                <button
                    on:click=move |_| is_open.update(|v| *v = !*v)
                    class="w-12 h-12 rounded-full bg-primary text-white shadow-lg hover:bg-primary-focus hover:shadow-xl transition-all duration-300 flex items-center justify-center group"
                    class:rotate-90=move || is_open.get()
                >
                    <i class="fa-solid fa-compass text-xl group-hover:scale-110 transition-transform"></i>
                </button>

                <Show when=move || is_open.get()>
                    <div class="absolute bottom-16 right-0 w-64 bg-white rounded-xl shadow-2xl border border-gray-100 overflow-hidden animate-fade-in-up">
                        <div class="p-3 border-b border-gray-100 bg-gradient-to-r from-primary/5 to-transparent">
                            <h4 class="font-semibold text-gray-800 flex items-center gap-2">
                                <i class="fa-solid fa-flag-checkered text-primary text-sm"></i>
                                "快速引导"
                            </h4>
                            <p class="text-xs text-gray-500 mt-0.5">"点击任意步骤开始体验"</p>
                        </div>
                        <div class="py-1">
                            <button
                                on:click=move |_| scroll_to_element(
                                    "voice-selector", 
                                    "声线选择指南", 
                                    "/videos/voice-guide.mp4",
                                    "在这里你可以选择不同的声线风格，包括温柔女声、磁性男声、童声等。点击试听按钮可以预览效果。",
                                    true  // 声线选择，使用安全顶部滚动
                                )
                                class="w-full px-3 py-2.5 flex items-center gap-3 hover:bg-gray-50 transition-colors text-left group/item"
                            >
                                <div class="w-7 h-7 rounded-full bg-blue-100 text-blue-600 flex items-center justify-center text-xs font-bold group-hover/item:scale-105 transition-transform">
                                    1
                                </div>
                                <div class="flex-1">
                                    <div class="text-sm font-medium text-gray-800">"声线选择"</div>
                                    <div class="text-xs text-gray-400">"挑选你喜欢的声音风格"</div>
                                </div>
                                <i class="fa-solid fa-chevron-right text-gray-300 text-xs group-hover/item:translate-x-0.5 transition-transform"></i>
                            </button>
                            
                            <button
                                on:click=move |_| scroll_to_element(
                                    "text-input",
                                    "文本输入教程",
                                    "/videos/text-input-guide.mov", 
                                    "在此输入你想要朗读的文字内容，支持中英文混排。最多可输入5000字符。",
                                    false  // 其他部分，使用居中滚动
                                )
                                class="w-full px-3 py-2.5 flex items-center gap-3 hover:bg-gray-50 transition-colors text-left group/item"
                            >
                                <div class="w-7 h-7 rounded-full bg-purple-100 text-purple-600 flex items-center justify-center text-xs font-bold group-hover/item:scale-105 transition-transform">
                                    2
                                </div>
                                <div class="flex-1">
                                    <div class="text-sm font-medium text-gray-800">"文本输入"</div>
                                    <div class="text-xs text-gray-400">"输入你想要朗读的内容"</div>
                                </div>
                                <i class="fa-solid fa-chevron-right text-gray-300 text-xs group-hover/item:translate-x-0.5 transition-transform"></i>
                            </button>
                            
                            <button
                                on:click=move |_| scroll_to_element(
                                    "param-control",
                                    "参数调节详解",
                                    "/videos/param-guide.mp4",
                                    "调整语速（0.5-2.0倍）、音调、音量。建议语速设为1.0，音调为0为最佳效果。",
                                    false  // 其他部分，使用居中滚动
                                )
                                class="w-full px-3 py-2.5 flex items-center gap-3 hover:bg-gray-50 transition-colors text-left group/item"
                            >
                                <div class="w-7 h-7 rounded-full bg-amber-100 text-amber-600 flex items-center justify-center text-xs font-bold group-hover/item:scale-105 transition-transform">
                                    3
                                </div>
                                <div class="flex-1">
                                    <div class="text-sm font-medium text-gray-800">"参数调节"</div>
                                    <div class="text-xs text-gray-400">"调音、语速、音量随心控"</div>
                                </div>
                                <i class="fa-solid fa-chevron-right text-gray-300 text-xs group-hover/item:translate-x-0.5 transition-transform"></i>
                            </button>
                            
                            <button
                                on:click=move |_| scroll_to_element(
                                    "audio-result",
                                    "输出结果操作指南",
                                    "/videos/result-guide.mp4",
                                    "试听生成的音频，下载MP3文件，或分享到社交媒体。支持多种音频格式导出。",
                                    false  // 其他部分，使用居中滚动
                                )
                                class="w-full px-3 py-2.5 flex items-center gap-3 hover:bg-gray-50 transition-colors text-left group/item rounded-b-xl"
                            >
                                <div class="w-7 h-7 rounded-full bg-green-100 text-green-600 flex items-center justify-center text-xs font-bold group-hover/item:scale-105 transition-transform">
                                    4
                                </div>
                                <div class="flex-1">
                                    <div class="text-sm font-medium text-gray-800">"输出结果"</div>
                                    <div class="text-xs text-gray-400">"试听、下载、分享你的作品"</div>
                                </div>
                                <i class="fa-solid fa-chevron-right text-gray-300 text-xs group-hover/item:translate-x-0.5 transition-transform"></i>
                            </button>
                        </div>
                    </div>
                </Show>
            </div>
        </Show>
    }
}

// 辅助函数
fn document() -> web_sys::Document {
    window().document().unwrap()
}

fn window() -> web_sys::Window {
    web_sys::window().unwrap()
}