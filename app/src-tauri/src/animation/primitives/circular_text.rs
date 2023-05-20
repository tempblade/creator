impl Animateable for AnimatedCircularText<'_> {
    fn draw(&mut self, mut canvas: &mut Canvas, timeline: &Timeline<'_>) {
        self.prepare(&mut canvas, &self.animation_data);

        self.sort_keyframes();

        self.paint.set_anti_alias(true);

        let default_text_typeface = &Typeface::default();

        let default_text_font = &Font::from_typeface(default_text_typeface, 190.0);

        let text_font: &Font = match self.font {
            Some(font) => font,
            None => default_text_font,
        };

        let radius: f32 = 0.35 * timeline.size.0.min(timeline.size.1) as f32;

        let mut path = Path::new();

        path.add_circle(
            (timeline.size.0 / 2, timeline.size.1 / 2),
            radius,
            PathDirection::CW,
        );

        let text_width = text_font.measure_str(self.text, Some(&self.paint));

        canvas.draw
    }

    fn sort_keyframes(&mut self) {
        self.origin.sort_keyframes();
    }
}
