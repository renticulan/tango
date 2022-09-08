use crate::graphics;

pub struct Backend<'a> {
    window: winit::window::Window,
    egui_ctx: egui::Context,
    painter: egui_wgpu::winit::Painter<'a>,
    egui_winit: egui_winit::State,
    shapes: Vec<egui::epaint::ClippedShape>,
    textures_delta: egui::TexturesDelta,
}

impl<'a> Backend<'a> {
    pub fn new<T>(
        window: winit::window::Window,
        mut painter: egui_wgpu::winit::Painter<'a>,
        event_loop: &winit::event_loop::EventLoopWindowTarget<T>,
    ) -> Self {
        unsafe {
            painter.set_window(Some(&window));
        }
        Self {
            window,
            painter,
            egui_ctx: egui::Context::default(),
            egui_winit: egui_winit::State::new(event_loop),
            shapes: vec![],
            textures_delta: egui::TexturesDelta::default(),
        }
    }
}

impl<'a> graphics::Backend for Backend<'a> {
    fn window(&self) -> &winit::window::Window {
        &self.window
    }

    fn paint(&mut self) {
        self.painter.paint_and_update_textures(
            self.egui_ctx.pixels_per_point(),
            egui::Rgba::BLACK,
            &self.egui_ctx.tessellate(std::mem::take(&mut self.shapes)),
            &std::mem::take(&mut self.textures_delta),
        );
    }

    fn egui_ctx(&self) -> &egui::Context {
        &self.egui_ctx
    }

    fn run(&mut self, mut run_ui: impl FnMut(&winit::window::Window, &egui::Context)) -> std::time::Duration {
        let egui::FullOutput {
            platform_output,
            repaint_after,
            textures_delta,
            shapes,
        } = self.egui_ctx.run(self.egui_winit.take_egui_input(&self.window), |ui| {
            run_ui(&self.window, ui)
        });

        self.egui_winit
            .handle_platform_output(&self.window, &self.egui_ctx, platform_output);

        self.shapes = shapes;
        self.textures_delta.append(textures_delta);
        repaint_after
    }

    fn on_window_event(&mut self, event: &winit::event::WindowEvent) -> bool {
        if let winit::event::WindowEvent::Resized(size) = event {
            self.painter.on_window_resized(size.width, size.height);
        }
        self.egui_winit.on_event(&self.egui_ctx, event)
    }
}
