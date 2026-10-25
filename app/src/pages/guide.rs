//! 新手引导（Spotlight Tour）
//!
//! 纯叠加层：不改动页面原有样式，只通过 `id` / `data-guide` 属性定位目标元素。
//! 服务端与水合阶段什么都不渲染（`mounted` 仅在客户端 Effect 中置为 true），
//! 所有 window / localStorage / 滚动 / 尺寸测量代码都被 `cfg(target_arch = "wasm32")` 隔离。

use leptos::prelude::*;
use leptos_router::hooks::use_location;

// ---------------------------------------------------------------------------
// 引导内容
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tour {
    /// 声音生成页（/generate）
    Generate,
    /// 网站导览（/home）
    Site,
}

struct Step {
    /// 依次尝试的 CSS 选择器，取第一个可见元素
    targets: &'static [&'static str],
    /// 目标不存在时直接跳过（否则居中显示）
    optional: bool,
    /// 顶栏里的胶囊元素：高亮圆角做成胶囊形，且不需要滚动
    pill: bool,
    icon: &'static str,
    title: &'static str,
    desc: &'static str,
    /// 目标不可见、居中显示时附加的说明
    fallback: Option<&'static str>,
}

const GENERATE_STEPS: &[Step] = &[
    Step {
        targets: &["#voice-selector"],
        optional: false,
        pill: false,
        icon: "fa-circle-user",
        title: "选择声线",
        desc: "先从声线库里挑一个喜欢的声音，点一下就能选中，接下来生成的音频都会用它来朗读。",
        fallback: None,
    },
    Step {
        targets: &["#text-input-section"],
        optional: false,
        pill: false,
        icon: "fa-comment",
        title: "输入文本",
        desc: "在这里写下想让它说的话。右上角可以「全屏编辑」长文本，右下角的 ✨ 按钮能让 AI 帮你改写、润色表达。",
        fallback: None,
    },
    Step {
        targets: &["#parameter-control"],
        optional: false,
        pill: false,
        icon: "fa-sliders",
        title: "调节参数",
        desc: "根据所选声线，拖动滑块调整音高、语速和音量，找到最合适的感觉；调好的配置还可以点「分享」发给朋友。",
        fallback: None,
    },
    Step {
        targets: &["[data-guide='param-mode']"],
        optional: true,
        pill: false,
        icon: "fa-comment-dots",
        title: "AI 语言控制",
        desc: "这个声线支持「语言控制」：用一句话描述想要的语气和情绪，比如“温柔一点，语速放慢”，AI 会照着说。两种参数可以同时生效。",
        fallback: None,
    },
    Step {
        // 生成成功后此按钮会变成「分享这段声音」，届时跳过本步
        targets: &["#generate-btn"],
        optional: true,
        pill: false,
        icon: "fa-wand-magic-sparkles",
        title: "生成音频",
        desc: "一切准备就绪后，点击「生成音频」，稍等片刻就能听到属于你的声音。",
        fallback: None,
    },
    Step {
        targets: &["#audio-result"],
        optional: false,
        pill: false,
        icon: "fa-volume-high",
        title: "收听与分享",
        desc: "生成完成后，可以在这里播放、查看声波、下载音频，还能点「分享这段声音」把作品发布到声音广场。",
        fallback: None,
    },
];

const SITE_STEPS: &[Step] = &[
    Step {
        targets: &["[data-guide='scenes']"],
        optional: true,
        pill: false,
        icon: "fa-layer-group",
        title: "从场景开始",
        desc: "欢迎来到耳朵！选择一个使用场景，我们会帮你准备合适的表达方式和声音风格。",
        fallback: None,
    },
    Step {
        targets: &["[data-guide='nav-generate']"],
        optional: false,
        pill: true,
        icon: "fa-wave-square",
        title: "声音生成",
        desc: "输入文字、选择声线、调整参数，一键把文字变成声音，是耳朵最核心的功能。",
        fallback: Some("入口地址：/generate"),
    },
    Step {
        targets: &["[data-guide='nav-voice']"],
        optional: false,
        pill: true,
        icon: "fa-music",
        title: "声音广场",
        desc: "听听大家分享的声音作品，遇到喜欢的可以直接使用同款配置。",
        fallback: Some("入口地址：/voice"),
    },
    Step {
        targets: &["[data-guide='nav-filters']"],
        optional: false,
        pill: true,
        icon: "fa-sliders",
        title: "声音滤镜",
        desc: "发现别人调好的声音滤镜，一键套用到你的文字上，也可以分享自己的滤镜。",
        fallback: Some("入口地址：/filters"),
    },
    Step {
        targets: &[],
        optional: false,
        pill: false,
        icon: "fa-wand-magic-sparkles",
        title: "渐进式配置",
        desc: "不熟悉参数也没关系：在「声音生成」页右上角进入「渐进式配置」，跟着提示一步步调出想要的声音。",
        fallback: Some("入口地址：/setup"),
    },
    Step {
        targets: &["[data-guide='nav-help']"],
        optional: false,
        pill: true,
        icon: "fa-circle-question",
        title: "帮助",
        desc: "遇到问题时来这里看看常见问答，也可以随时重新查看新手引导。",
        fallback: Some("入口地址：/help"),
    },
    Step {
        targets: &["[data-guide='account']"],
        optional: false,
        pill: true,
        icon: "fa-user",
        title: "登录 / 头像",
        desc: "登录后可以保存和发布作品；之后点击右上角头像，就能进入个人主页和设置。",
        fallback: None,
    },
];

impl Tour {
    fn steps(self) -> &'static [Step] {
        match self {
            Tour::Generate => GENERATE_STEPS,
            Tour::Site => SITE_STEPS,
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn storage_key(self) -> &'static str {
        match self {
            Tour::Generate => "eardo.guide.v1.generate",
            Tour::Site => "eardo.guide.v1.site",
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn param(self) -> &'static str {
        match self {
            Tour::Generate => "generate",
            Tour::Site => "site",
        }
    }

    fn for_path(path: &str) -> Option<Tour> {
        match path.trim_end_matches('/') {
            "/generate" => Some(Tour::Generate),
            "/home" => Some(Tour::Site),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// 状态
// ---------------------------------------------------------------------------

/// 视口坐标下的矩形
#[derive(Clone, Copy, PartialEq, Debug, Default)]
struct Rect {
    top: f64,
    left: f64,
    width: f64,
    height: f64,
}

#[derive(Clone, Copy)]
struct Guide {
    tour: RwSignal<Option<Tour>>,
    /// 本次实际展示的步骤（跳过了缺失的可选步骤）
    plan: RwSignal<Vec<usize>>,
    /// 当前在 plan 中的位置
    pos: RwSignal<usize>,
    /// 当前目标的位置；None = 居中显示
    rect: RwSignal<Option<Rect>>,
    /// 视口宽高
    viewport: RwSignal<(f64, f64)>,
    /// 悬浮按钮是否需要避让（例如与播放器控制栏重叠）
    fab_hidden: RwSignal<bool>,
    /// 页面正在滚动：此时高亮框直接跟随，不做过渡动画（否则会“追赶”滚动而滞后）
    scrolling: RwSignal<bool>,
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    scroll_seq: StoredValue<u32>,
}

impl Guide {
    fn new() -> Self {
        Self {
            tour: RwSignal::new(None),
            plan: RwSignal::new(Vec::new()),
            pos: RwSignal::new(0),
            rect: RwSignal::new(None),
            viewport: RwSignal::new((1280.0, 800.0)),
            fab_hidden: RwSignal::new(false),
            scrolling: RwSignal::new(false),
            scroll_seq: StoredValue::new(0),
        }
    }

    fn current_step(&self) -> Option<&'static Step> {
        let tour = self.tour.get()?;
        let pos = self.pos.get();
        let idx = self.plan.with(|p| p.get(pos).copied())?;
        tour.steps().get(idx)
    }

    fn current_step_untracked(&self) -> Option<&'static Step> {
        let tour = self.tour.get_untracked()?;
        let pos = self.pos.get_untracked();
        let idx = self.plan.with_untracked(|p| p.get(pos).copied())?;
        tour.steps().get(idx)
    }

    fn start(&self, tour: Tour) {
        let plan: Vec<usize> = tour
            .steps()
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.optional || dom::find(s.targets).is_some())
            .map(|(i, _)| i)
            .collect();
        if plan.is_empty() {
            return;
        }
        self.plan.set(plan);
        self.pos.set(0);
        self.rect.set(None);
        self.tour.set(Some(tour));
        self.show_step();
    }

    fn next(&self) {
        let len = self.plan.with_untracked(|p| p.len());
        let pos = self.pos.get_untracked();
        if pos + 1 >= len {
            self.close(true);
        } else {
            self.pos.set(pos + 1);
            self.show_step();
        }
    }

    fn prev(&self) {
        let pos = self.pos.get_untracked();
        if pos > 0 {
            self.pos.set(pos - 1);
            self.show_step();
        }
    }

    /// 关闭引导并记住（完成 / 跳过都不再自动弹出）
    fn close(&self, completed: bool) {
        if let Some(tour) = self.tour.get_untracked() {
            dom::storage_set(tour.storage_key(), if completed { "done" } else { "skipped" });
        }
        self.tour.set(None);
        self.rect.set(None);
        dom::focus_later("eardo-guide-fab");
    }

    /// 离开页面时静默关闭（不记录）
    fn dismiss(&self) {
        if self.tour.get_untracked().is_some() {
            self.tour.set(None);
            self.rect.set(None);
        }
    }

    fn show_step(&self) {
        if let Some(step) = self.current_step_untracked() {
            if !step.pill {
                dom::scroll_to_target(step.targets);
            }
        }
        self.measure();
        dom::focus_later("eardo-guide-next");
    }

    /// 重新测量视口与目标位置（滚动 / 缩放 / 布局变化时调用）
    fn measure(&self) {
        let vp = dom::viewport();
        if self.viewport.get_untracked() != vp {
            self.viewport.set(vp);
        }
        if let Some(step) = self.current_step_untracked() {
            let rect = dom::find(step.targets).and_then(|el| dom::rect_of(&el));
            if self.rect.get_untracked() != rect {
                self.rect.set(rect);
            }
        } else {
            let hidden = dom::fab_overlaps_avoid_zone();
            if self.fab_hidden.get_untracked() != hidden {
                self.fab_hidden.set(hidden);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// DOM 辅助（仅浏览器）
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
mod dom {
    use super::Rect;
    use wasm_bindgen::JsCast;

    fn document() -> Option<web_sys::Document> {
        web_sys::window().and_then(|w| w.document())
    }

    pub fn viewport() -> (f64, f64) {
        document()
            .and_then(|d| d.document_element())
            .map(|el| (el.client_width() as f64, el.client_height() as f64))
            .unwrap_or((1280.0, 800.0))
    }

    pub fn rect_of(el: &web_sys::Element) -> Option<Rect> {
        let r = el.get_bounding_client_rect();
        (r.width() > 0.0 && r.height() > 0.0).then(|| Rect {
            top: r.top(),
            left: r.left(),
            width: r.width(),
            height: r.height(),
        })
    }

    pub fn find(selectors: &[&str]) -> Option<web_sys::Element> {
        let doc = document()?;
        selectors.iter().find_map(|sel| {
            doc.query_selector(sel)
                .ok()
                .flatten()
                .filter(|el| rect_of(el).is_some())
        })
    }

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    pub fn storage_get(key: &str) -> Option<String> {
        storage()?.get_item(key).ok().flatten()
    }

    pub fn storage_set(key: &str, value: &str) {
        if let Some(s) = storage() {
            let _ = s.set_item(key, value);
        }
    }

    pub fn reduced_motion() -> bool {
        web_sys::window()
            .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok().flatten())
            .is_some_and(|m| m.matches())
    }

    pub fn focus_later(id: &'static str) {
        leptos::prelude::request_animation_frame(move || {
            if let Some(el) = document()
                .and_then(|d| d.get_element_by_id(id))
                .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
            {
                let _ = el.focus();
            }
        });
    }

    /// 把目标滚到视口里合适的位置（避开吸顶导航栏；窄屏时避开底部弹出的卡片）
    pub fn scroll_to_target(selectors: &[&str]) {
        let Some(window) = web_sys::window() else { return };
        let Some(r) = find(selectors).and_then(|el| rect_of(&el)) else { return };
        let (vw, vh) = viewport();
        let header = 88.0;
        let narrow = vw < 640.0;
        let visible_bottom = if narrow { vh * 0.55 } else { vh - 24.0 };
        let needs = r.top < header || r.top + r.height.min(vh * 0.5) > visible_bottom;
        if !needs {
            return;
        }
        let y = window.scroll_y().unwrap_or(0.0);
        let far = (r.top - header).abs() > vh * 1.5;
        let tall = r.height > (vh - header) * 0.7;
        let target = if narrow || tall {
            y + r.top - header
        } else {
            (y + r.top + r.height / 2.0 - vh / 2.0).min(y + r.top - header)
        };
        let opts = web_sys::ScrollToOptions::new();
        opts.set_top(target.max(0.0));
        // 距离很远（例如窄屏下很长的声线列表）时直接跳转，避免长时间滚动
        opts.set_behavior(if far || reduced_motion() {
            web_sys::ScrollBehavior::Auto
        } else {
            web_sys::ScrollBehavior::Smooth
        });
        window.scroll_to_with_scroll_to_options(&opts);
    }

    /// 悬浮按钮是否压在了需要避让的元素（如播放器控制栏）上
    pub fn fab_overlaps_avoid_zone() -> bool {
        let Some(doc) = document() else { return false };
        let (vw, vh) = viewport();
        // 悬浮按钮所在区域（右下角，按最大尺寸估算，避免显隐时抖动）
        let zone = Rect {
            top: vh - 80.0,
            left: vw - 180.0,
            width: 180.0,
            height: 80.0,
        };
        let Ok(list) = doc.query_selector_all("[data-guide-avoid]") else { return false };
        (0..list.length())
            .filter_map(|i| list.item(i))
            .filter_map(|n| n.dyn_into::<web_sys::Element>().ok())
            .filter_map(|el| rect_of(&el))
            .any(|r| {
                r.left < zone.left + zone.width
                    && r.left + r.width > zone.left
                    && r.top < zone.top + zone.height
                    && r.top + r.height > zone.top
            })
    }

    /// 弹窗内的 Tab 焦点循环
    pub fn trap_tab(ev: &web_sys::KeyboardEvent) {
        let Some(doc) = document() else { return };
        let Ok(list) = doc.query_selector_all("#eardo-guide-pop button") else { return };
        let buttons: Vec<web_sys::HtmlElement> = (0..list.length())
            .filter_map(|i| list.item(i))
            .filter_map(|n| n.dyn_into::<web_sys::HtmlElement>().ok())
            .collect();
        let (Some(first), Some(last)) = (buttons.first(), buttons.last()) else { return };
        let active = doc.active_element();
        let inside = active
            .as_ref()
            .is_some_and(|a| buttons.iter().any(|b| b.unchecked_ref::<web_sys::Element>() == a));
        let at = |b: &web_sys::HtmlElement| {
            active.as_ref().is_some_and(|a| b.unchecked_ref::<web_sys::Element>() == a)
        };
        if !inside {
            ev.prevent_default();
            let _ = first.focus();
        } else if ev.shift_key() && at(first) {
            ev.prevent_default();
            let _ = last.focus();
        } else if !ev.shift_key() && at(last) {
            ev.prevent_default();
            let _ = first.focus();
        }
    }
}

/// 服务端占位：引导只在浏览器中运行
#[cfg(not(target_arch = "wasm32"))]
mod dom {
    use super::Rect;

    pub struct Element;

    pub fn viewport() -> (f64, f64) {
        (1280.0, 800.0)
    }
    pub fn rect_of(_: &Element) -> Option<Rect> {
        None
    }
    pub fn find(_: &[&str]) -> Option<Element> {
        None
    }
    pub fn storage_set(_: &str, _: &str) {}
    pub fn focus_later(_: &'static str) {}
    pub fn scroll_to_target(_: &[&str]) {}
    pub fn fab_overlaps_avoid_zone() -> bool {
        false
    }
}

// ---------------------------------------------------------------------------
// 布局计算
// ---------------------------------------------------------------------------

const POP_MAX_W: f64 = 360.0;
const EDGE: f64 = 16.0;
const GAP: f64 = 14.0;

/// 高亮框：目标外扩并裁剪到视口内
fn spotlight_rect(r: Rect, (vw, vh): (f64, f64), pill: bool) -> Rect {
    let pad = if pill { 6.0 } else { 8.0 };
    let top = (r.top - pad).max(4.0);
    let left = (r.left - pad).max(4.0);
    let bottom = (r.top + r.height + pad).min(vh - 4.0);
    let right = (r.left + r.width + pad).min(vw - 4.0);
    Rect {
        top,
        left,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    }
}

/// 弹出卡片的定位样式
fn popover_style(rect: Option<Rect>, (vw, vh): (f64, f64), pill: bool) -> String {
    // 窄屏：固定停靠在底部
    if vw < 640.0 {
        return "left:12px;right:12px;bottom:12px;".to_string();
    }
    let w = POP_MAX_W.min(vw - 2.0 * EDGE);
    let Some(r) = rect.map(|r| spotlight_rect(r, (vw, vh), pill)) else {
        return format!("width:{w}px;left:50%;top:50%;transform:translate(-50%,-50%);");
    };
    let need = 230.0;
    let below = vh - (r.top + r.height);
    let above = r.top;
    let centered_left = (r.left + r.width / 2.0 - w / 2.0).clamp(EDGE, (vw - w - EDGE).max(EDGE));

    if below >= need {
        format!("width:{w}px;left:{centered_left}px;top:{}px;", r.top + r.height + GAP)
    } else if above >= need + 40.0 {
        format!("width:{w}px;left:{centered_left}px;bottom:{}px;", vh - r.top + GAP)
    } else {
        let top = r.top.clamp(96.0, (vh - 280.0).max(96.0));
        if vw - (r.left + r.width) >= w + 2.0 * GAP {
            format!("width:{w}px;left:{}px;top:{top}px;", r.left + r.width + GAP)
        } else if r.left >= w + 2.0 * GAP {
            format!("width:{w}px;left:{}px;top:{top}px;", r.left - w - GAP)
        } else {
            format!("width:{w}px;left:{centered_left}px;bottom:24px;")
        }
    }
}

// ---------------------------------------------------------------------------
// 组件
// ---------------------------------------------------------------------------

/// 全局挂载一次（Router 内）。根据当前路由决定是否提供引导。
#[component]
pub fn GuideHost() -> impl IntoView {
    let guide = Guide::new();
    let location = use_location();
    let mounted = RwSignal::new(false);

    // 仅在客户端水合完成后才挂载，保证 SSR 与水合结构一致
    Effect::new(move |_| mounted.set(true));

    let current_tour = Memo::new(move |_| Tour::for_path(&location.pathname.get()));

    // 路由变化：关闭旧引导；首次访问自动开始（或 ?guide=xxx 强制开始）
    Effect::new(move |_| {
        let tour = current_tour.get();
        guide.dismiss();
        #[cfg(target_arch = "wasm32")]
        if let Some(tour) = tour {
            let forced = location
                .query
                .with_untracked(|q| q.get("guide").is_some_and(|v| v == tour.param()));
            let seen = dom::storage_get(tour.storage_key()).is_some();
            if forced || !seen {
                // 稍等页面内容（声线列表等）渲染完成
                set_timeout(
                    move || {
                        if current_tour.get_untracked() == Some(tour)
                            && guide.tour.get_untracked().is_none()
                        {
                            guide.start(tour);
                        }
                    },
                    std::time::Duration::from_millis(700),
                );
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = tour;
    });

    // 监听键盘 / 滚动 / 缩放，并定时重新测量（异步加载导致的布局变化）
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        use leptos::ev;

        let keys = window_event_listener(ev::keydown, move |e: web_sys::KeyboardEvent| {
            if guide.tour.get_untracked().is_none() {
                return;
            }
            match e.key().as_str() {
                "Escape" => {
                    e.prevent_default();
                    guide.close(false);
                }
                "ArrowRight" => {
                    e.prevent_default();
                    guide.next();
                }
                "ArrowLeft" => {
                    e.prevent_default();
                    guide.prev();
                }
                "Tab" => dom::trap_tab(&e),
                _ => {}
            }
        });
        let resize = window_event_listener(ev::resize, move |_| guide.measure());
        let scroll = window_event_listener(ev::scroll, move |_| {
            guide.measure();
            if !guide.scrolling.get_untracked() {
                guide.scrolling.set(true);
            }
            let seq = guide.scroll_seq.get_value().wrapping_add(1);
            guide.scroll_seq.set_value(seq);
            set_timeout(
                move || {
                    if guide.scroll_seq.get_value() == seq {
                        guide.scrolling.set(false);
                        guide.measure();
                    }
                },
                std::time::Duration::from_millis(160),
            );
        });
        let interval = set_interval_with_handle(
            move || guide.measure(),
            std::time::Duration::from_millis(300),
        )
        .ok();
        on_cleanup(move || {
            keys.remove();
            resize.remove();
            scroll.remove();
            if let Some(h) = interval {
                h.clear();
            }
        });
    });

    view! {
        <Show when=move || mounted.get()>
            // 悬浮的「新手引导」按钮：只在有引导的页面、引导未进行时出现
            <Show when=move || current_tour.get().is_some() && guide.tour.get().is_none()>
                <div class="fixed bottom-5 right-5 z-[60] animate-fade-in">
                    <button
                        id="eardo-guide-fab"
                        type="button"
                        aria-label="新手引导"
                        title="新手引导"
                        class="inline-flex items-center justify-center gap-2 h-11 w-11 sm:w-auto sm:px-4 rounded-full bg-white/90 backdrop-blur-sm border border-primary/20 shadow-soft hover:shadow-hover text-gray-600 hover:text-primary motion-safe:transition-all motion-safe:duration-300 motion-safe:hover:-translate-y-0.5 focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
                        class:opacity-0=move || guide.fab_hidden.get()
                        class:pointer-events-none=move || guide.fab_hidden.get()
                        tabindex=move || if guide.fab_hidden.get() { "-1" } else { "0" }
                        aria-hidden=move || if guide.fab_hidden.get() { "true" } else { "false" }
                        on:click=move |_| {
                            if let Some(tour) = current_tour.get_untracked() {
                                guide.start(tour);
                            }
                        }
                    >
                        <i class="fa-solid fa-compass text-primary text-lg"></i>
                        <span class="hidden sm:inline text-sm font-medium">"新手引导"</span>
                    </button>
                </div>
            </Show>

            <Show when=move || guide.tour.get().is_some()>
                <GuideOverlay guide=guide />
            </Show>
        </Show>
    }
}

#[component]
fn GuideOverlay(guide: Guide) -> impl IntoView {
    let pill = move || guide.current_step().is_some_and(|s| s.pill);

    let spotlight = move || {
        guide
            .rect
            .get()
            .map(|r| spotlight_rect(r, guide.viewport.get(), pill()))
    };

    let total = move || guide.plan.with(|p| p.len());
    let is_last = move || guide.pos.get() + 1 >= total();

    view! {
        <div class="fixed inset-0 z-[1000]">
            // 点击拦截层（引导期间不误触页面）；没有目标时同时负责变暗
            <div
                class="absolute inset-0 motion-safe:transition-colors motion-safe:duration-300"
                style=move || {
                    if spotlight().is_none() { "background-color:rgba(17,24,39,.55);" } else { "background-color:transparent;" }
                }
            ></div>

            // 高亮框：外围用超大阴影形成遮罩，留出目标区域（常驻节点，位置变化时平滑过渡）
            <div
                class=move || {
                    if guide.scrolling.get() {
                        "absolute pointer-events-none"
                    } else {
                        "absolute pointer-events-none motion-safe:transition-all motion-safe:duration-300 motion-safe:ease-out"
                    }
                }
                style=move || {
                    let radius = if pill() { "9999px" } else { "18px" };
                    match spotlight() {
                        Some(r) => format!(
                            "top:{}px;left:{}px;width:{}px;height:{}px;border-radius:{radius};\
                             box-shadow:0 0 0 3px rgba(251,191,36,.95),0 0 0 8px rgba(251,191,36,.25),0 0 0 9999px rgba(17,24,39,.55);",
                            r.top, r.left, r.width, r.height,
                        ),
                        None => "opacity:0;top:50%;left:50%;width:0;height:0;".to_string(),
                    }
                }
            ></div>

            // 说明卡片
            <div
                class=move || {
                    if guide.scrolling.get() {
                        "absolute"
                    } else {
                        "absolute motion-safe:transition-[top,left,bottom] motion-safe:duration-300 motion-safe:ease-out"
                    }
                }
                style=move || popover_style(guide.rect.get(), guide.viewport.get(), pill())
            >
                {move || {
                    let step = guide.current_step()?;
                    let pos = guide.pos.get();
                    let n = total();
                    let fallback = step.fallback;
                    Some(view! {
                        <div
                            id="eardo-guide-pop"
                            role="dialog"
                            aria-modal="true"
                            aria-labelledby="eardo-guide-title"
                            aria-describedby="eardo-guide-desc"
                            class="bg-white rounded-2xl shadow-hover border border-primary/20 p-5 text-dark motion-safe:animate-guide-pop"
                        >
                            <div class="flex items-center justify-between mb-3">
                                <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full bg-primary/10 text-primary-focus text-xs font-semibold">
                                    <i class="fa-solid fa-compass"></i>
                                    "新手引导"
                                </span>
                                <span class="text-xs text-gray-400 tabular-nums" aria-live="polite">
                                    {format!("{} / {}", pos + 1, n)}
                                </span>
                            </div>

                            <h3 id="eardo-guide-title" class="text-lg font-semibold flex items-center mb-1.5">
                                <i class=format!("fa-solid {} text-primary mr-2", step.icon)></i>
                                {step.title}
                            </h3>
                            <p id="eardo-guide-desc" class="text-sm text-gray-600 leading-relaxed">
                                {step.desc}
                            </p>
                            {move || {
                                fallback
                                    .filter(|_| guide.rect.with(|r| r.is_none()))
                                    .map(|note| {
                                        view! {
                                            <p class="mt-2 text-xs text-gray-400 flex items-center">
                                                <i class="fa-solid fa-location-arrow mr-1.5 text-primary/70"></i>
                                                {note}
                                            </p>
                                        }
                                    })
                            }}

                            // 进度
                            <div class="mt-4 flex items-center gap-1.5" aria-hidden="true">
                                {(0..n)
                                    .map(|i| {
                                        let cls = if i == pos {
                                            "h-1.5 w-5 rounded-full bg-primary"
                                        } else if i < pos {
                                            "h-1.5 w-1.5 rounded-full bg-primary/40"
                                        } else {
                                            "h-1.5 w-1.5 rounded-full bg-gray-200"
                                        };
                                        view! { <span class=cls></span> }
                                    })
                                    .collect::<Vec<_>>()}
                            </div>

                            <div class="mt-4 flex items-center justify-between gap-2">
                                {(pos + 1 < n)
                                    .then(|| {
                                        view! {
                                            <button
                                                type="button"
                                                class="text-sm text-gray-400 hover:text-dark px-2 py-1.5 -ml-2 rounded-lg transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
                                                on:click=move |_| guide.close(false)
                                            >
                                                "跳过"
                                            </button>
                                        }
                                    })}
                                <div class="ml-auto flex items-center gap-2">
                                    {(pos > 0)
                                        .then(|| {
                                            view! {
                                                <button
                                                    type="button"
                                                    class="px-3.5 py-2 rounded-lg border border-gray-200 text-sm text-gray-600 hover:bg-gray-50 transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-primary/60"
                                                    on:click=move |_| guide.prev()
                                                >
                                                    "上一步"
                                                </button>
                                            }
                                        })}
                                    <button
                                        id="eardo-guide-next"
                                        type="button"
                                        class="px-4 py-2 rounded-lg bg-primary hover:bg-primary-focus text-white text-sm font-medium shadow-sm hover:shadow transition-all active:scale-[0.98] focus:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-primary"
                                        on:click=move |_| guide.next()
                                    >
                                        {if is_last() { "完成" } else { "下一步" }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    })
                }}
            </div>
        </div>
    }
}

/// 帮助中心里的「重新查看新手引导」入口
#[component]
pub fn GuideReplayCard() -> impl IntoView {
    use leptos_router::components::A;
    view! {
        <div class="mt-6 bg-white rounded-lg border border-gray-200 shadow-sm hover:shadow-md transition-shadow duration-300 px-6 py-5 md:px-8 flex flex-col md:flex-row md:items-center gap-4">
            <div class="flex items-start gap-3 flex-1">
                <div class="w-10 h-10 rounded-full bg-primary/10 text-primary flex items-center justify-center shrink-0">
                    <i class="fa-solid fa-compass text-lg"></i>
                </div>
                <div>
                    <h3 class="text-lg md:text-xl font-semibold text-gray-800">"重新查看新手引导"</h3>
                    <p class="text-sm text-gray-500 mt-1">"跟着引导再走一遍，快速熟悉各个功能的位置和用法。"</p>
                </div>
            </div>
            <div class="flex gap-3 flex-wrap">
                <A
                    href="/generate?guide=generate"
                    attr:class="px-5 py-2 bg-primary hover:bg-primary/90 text-white rounded-lg transition-colors duration-300 font-medium text-sm"
                >
                    "声音生成引导"
                </A>
                <A
                    href="/home?guide=site"
                    attr:class="px-5 py-2 border border-primary text-primary hover:bg-primary/5 rounded-lg transition-colors duration-300 font-medium text-sm"
                >
                    "网站导览"
                </A>
            </div>
        </div>
    }
}
