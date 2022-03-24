use std::any::TypeId;
use std::borrow::BorrowMut;
use std::pin::Pin;

use azul::prelude::*;
use azul::str::String as AzString;
use azul::widgets::{Button, Label};
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

	"Hello Asteracea!"
}
