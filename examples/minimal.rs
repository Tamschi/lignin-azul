use azul::prelude::*;
use std::{
	any::TypeId,
	pin::Pin,
	sync::atomic::{AtomicU32, Ordering},
};
use tap::Pipe;

extern "C" fn layout(data: &mut RefAny, _callback_info: &mut LayoutCallbackInfo) -> StyledDom {
	let bump = asteracea::bumpalo::Bump::new();

	let vdom = data
		.downcast_ref::<Pin<Box<TestApp>>>()
		.unwrap()
		.as_ref()
		.render(&bump, TestAppRenderArgs::builder().build())
		.unwrap();

	let mut body = Dom::body().style(Css::empty());
	lignin_azul::render(&vdom, &mut body, 1000).unwrap();
	body
}

fn main() {
	let app = TestApp::new(
		rhizome::sync::Node::new(TypeId::of::<()>()).as_ref(),
		TestAppNewArgs::builder().build(),
	)
	.unwrap()
	.pipe(Box::pin);
	let app = App::new(RefAny::new(app), AppConfig::new(LayoutSolver::Default));
	let window = WindowCreateOptions::new(layout);
	app.run(window);
}

asteracea::component! {
	TestApp()() -> !Sync

	let self.counter = AtomicU32::new(0);
	[
		<button "Hello Button!"
			on bubble click = fn(self, _event) {
				self.counter.fetch_add(1, Ordering::Relaxed);
				lignin_azul::UPDATE.store(true, Ordering::Relaxed);
			}
		>
		!"The button was clicked {} times."(self.counter.load(Ordering::Relaxed))
	]
}
