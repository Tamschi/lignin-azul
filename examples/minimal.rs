use asteracea::services::Invalidator;
use azul::prelude::*;
use debugless_unwrap::DebuglessUnwrap;
use std::{
	any::TypeId,
	pin::Pin,
	sync::atomic::{AtomicU32, Ordering},
	task::Context,
};
use tap::Pipe;
use this_is_fine::FineExt;

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
	let resource_root = rhizome::sync::Node::new(TypeId::of::<()>());
	use rhizome::sync::Inject;
	<dyn Invalidator>::inject(
		resource_root.as_ref(),
		|on_presented: Option<&mut Context>| {
			if let Some(on_presented) = on_presented {
				lignin_azul::ON_PRESENTED
					.lock()
					.unwrap()
					.push(on_presented.waker().clone())
			}
			lignin_azul::UPDATE.store(true, Ordering::Relaxed);
		},
	)
	.not_fine()
	.debugless_unwrap();

	let app = TestApp::new(resource_root.as_ref(), TestAppNewArgs::builder().build())
		.unwrap()
		.pipe(Box::pin);
	let app = App::new(RefAny::new(app), AppConfig::new(LayoutSolver::Default));
	let window = WindowCreateOptions::new(layout);
	app.run(window);
}

asteracea::component! {
	TestApp(
		priv dyn invalidator: dyn Invalidator,
	)() -> !Sync

	let self.counter = AtomicU32::new(0);
	[
		<button "Hello Button!"
			on bubble click = fn(self, _event) {
				self.counter.fetch_add(1, Ordering::Relaxed);
				self.invalidator.invalidate_with_context(None);
			}
		>
		!"The button was clicked {} times."(self.counter.load(Ordering::Relaxed))
	]
}
