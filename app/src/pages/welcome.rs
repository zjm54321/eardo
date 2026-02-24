#[cfg(target_arch = "wasm32")]
use leptos::ev;
use leptos::prelude::*;
use leptos_router::components::A;
use std::time::Duration;

#[component]
pub fn WelcomePage() -> impl IntoView {
    // Current step: 0 (Logo), 1 (Slogan), 2 (Button)
    let (step, set_step) = signal(0);
    let (in_cooldown, set_in_cooldown) = signal(false);

    // Scroll handler logic
    #[allow(unused_variables)]
    let handle_scroll = move |delta_y: f64| {
        if in_cooldown.get() {
            return;
        }

        let current = step.get();
        if delta_y > 0.0 {
            // Scroll Down -> Next Step
            if current < 2 {
                set_step.set(current + 1);
                set_in_cooldown.set(true);
                set_timeout(
                    move || set_in_cooldown.set(false),
                    Duration::from_millis(800),
                );
            }
        } else if delta_y < 0.0 {
            // Scroll Up -> Prev Step
            if current > 0 {
                set_step.set(current - 1);
                set_in_cooldown.set(true);
                set_timeout(
                    move || set_in_cooldown.set(false),
                    Duration::from_millis(800),
                );
            }
        }
    };

    // Desktop: Wheel Event Listener
    Effect::new(move |_| {
        // This code block only compiles for WASM (browser), preventing server-side errors
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::leptos_dom::helpers::window_event_listener;
            let handle = window_event_listener(ev::wheel, move |e: ev::WheelEvent| {
                handle_scroll(e.delta_y());
            });
            on_cleanup(move || handle.remove());
        }
    });

    // Mobile: Touch Event Listeners
    #[allow(unused_variables)]
    let (touch_start_y, set_touch_start_y) = signal(0.0);

    Effect::new(move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            use leptos::leptos_dom::helpers::window_event_listener;

            // Touch Start
            let h1 = window_event_listener(ev::touchstart, move |e: ev::TouchEvent| {
                if let Some(touch) = e.touches().item(0) {
                    set_touch_start_y.set(touch.client_y() as f64);
                }
            });

            // Touch End
            let h2 = window_event_listener(ev::touchend, move |e: ev::TouchEvent| {
                if let Some(touch) = e.changed_touches().item(0) {
                    let end_y = touch.client_y() as f64;
                    let start_y = touch_start_y.get();
                    // Explicit type annotation to fix E0689
                    let diff: f64 = start_y - end_y;

                    if diff.abs() > 50.0 {
                        handle_scroll(diff);
                    }
                }
            });

            on_cleanup(move || {
                h1.remove();
                h2.remove();
            });
        }
    });

    // --- Computed Styles (Fixes IntoClass errors) ---
    let is_step_0 = Signal::derive(move || step.get() == 0);
    let is_step_1 = Signal::derive(move || step.get() == 1);
    let is_step_2 = Signal::derive(move || step.get() == 2);
    let is_step_ge_1 = Signal::derive(move || step.get() >= 1);
    let is_step_lt_1 = Signal::derive(move || step.get() < 1);
    let is_step_ge_2 = Signal::derive(move || step.get() >= 2);
    let is_step_lt_2 = Signal::derive(move || step.get() < 2);

    // Helper for indicator dots
    let is_step_eq_i = |i: i32| Signal::derive(move || step.get() == i);
    let is_step_neq_i = |i: i32| Signal::derive(move || step.get() != i);

    view! {
        <div class="relative w-full h-screen overflow-hidden bg-gradient-summer">

            <div class="absolute inset-0 flex flex-col items-center justify-center">

                // 1. Logo and Title
                <div
                    class="text-center z-10 flex flex-col items-center transition-all duration-1000 ease-in-out"
                    class:translate-y-0=is_step_0
                    class:-translate-y-24=is_step_1
                    class:-translate-y-32=is_step_2
                    class:scale-100=is_step_0
                    class:scale-90=is_step_ge_1
                >
                    <div class="mb-6 relative inline-block">
                        <img
                            src="/logo.png"
                            alt="Eardo Logo"
                            class="w-32 h-32 md:w-48 md:h-48 lg:w-64 lg:h-64 object-contain drop-shadow-lg transition-all duration-1000"
                        />
                        <div class="absolute inset-0 bg-primary/20 blur-3xl rounded-full -z-10 animate-pulse"></div>
                    </div>

                    <h1 class="text-5xl md:text-7xl font-black tracking-tight text-transparent bg-clip-text bg-gradient-to-r from-primary to-secondary mb-2 drop-shadow-sm transition-all duration-1000">
                        "白昼聆夏"
                    </h1>
                    <p class="text-gray-400 text-lg md:text-xl font-medium tracking-[0.5em] uppercase transition-all duration-1000">
                        "EARDO"
                    </p>
                </div>

                // 2. Slogan
                <div
                    class="absolute top-[60%] text-center max-w-2xl px-6 transition-all duration-1000 ease-in-out transform"
                    class:opacity-0=is_step_lt_1
                    class:translate-y-8=is_step_lt_1
                    class:opacity-100=is_step_ge_1
                    class:translate-y-0=is_step_ge_1
                    class:-translate-y-8=is_step_2
                >
                    <h2 class="text-3xl md:text-5xl font-bold text-dark leading-tight text-shadow">
                        "声音也能如此多彩"
                    </h2>
                    <div class="w-24 h-1.5 bg-gradient-to-r from-primary to-secondary rounded-full mx-auto mt-6"></div>
                </div>

                // 3. Button
                <div
                    class="absolute top-[80%] transition-all duration-1000 ease-in-out transform"
                    class:opacity-0=is_step_lt_2
                    class:translate-y-8=is_step_lt_2
                    class:scale-90=is_step_lt_2
                    class:opacity-100=is_step_ge_2
                    class:translate-y-0=is_step_ge_2
                    class:scale-100=is_step_ge_2
                    class:pointer-events-none=is_step_lt_2
                    class:pointer-events-auto=is_step_ge_2
                >
                    <A href="/home" attr:class="group relative inline-flex items-center justify-center px-10 py-5 text-xl font-bold text-white transition-all duration-300 bg-gradient-to-r from-primary to-secondary rounded-full shadow-lg hover:shadow-2xl hover:-translate-y-1 focus:outline-none focus:ring-4 focus:ring-primary/30 overflow-hidden">
                        <span class="relative z-10 flex items-center">
                            "开启我的旅程"
                            <i class="fa-solid fa-arrow-right ml-3 group-hover:translate-x-1 transition-transform"></i>
                        </span>
                        <div class="absolute inset-0 -translate-x-full group-hover:translate-x-0 bg-white/20 transition-transform duration-500 skew-x-12"></div>
                    </A>
                </div>

            </div>

            // Scroll Hint
            <div
                class="fixed bottom-10 left-0 right-0 text-center transition-opacity duration-500"
                class:opacity-100=is_step_lt_2
                class:opacity-0=is_step_ge_2
            >
                <button
                    class="focus:outline-none animate-bounce cursor-pointer bg-transparent border-none"
                    on:click=move |_| {
                        if step.get() < 2 {
                            set_step.update(|s| *s += 1);
                        }
                    }
                >
                    <i class="fa-solid fa-chevron-down text-primary text-2xl"></i>
                </button>
            </div>

            // Indicators
            <div class="fixed right-8 top-1/2 transform -translate-y-1/2 flex flex-col gap-4 z-20">
                {(0..3).map(|i| {
                    view! {
                        <button
                            class="w-3 h-3 rounded-full transition-all duration-500 border border-primary/50"
                            class:bg-primary=is_step_eq_i(i)
                            class:bg-transparent=is_step_neq_i(i)
                            class:scale-125=is_step_eq_i(i)
                            on:click=move |_| set_step.set(i)
                        />
                    }
                }).collect::<Vec<_>>()}
            </div>

        </div>
    }
}
