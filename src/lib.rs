//! TODO_DOCS_DESCRIPTION
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Flignin-azul)](https://iteration-square.schichler.dev/#narrow/stream/project.2Flignin-azul)

#![doc(html_root_url = "https://docs.rs/lignin-azul/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)] // TODO

use azul::{
	callbacks::{CallbackInfo, RefAny, Update},
	dom::Dom,
	prelude::{Css, StyledDom},
	str::String as AzString,
	widgets::Button,
};
use lignin::{CallbackRef, Element, EventBinding, Node, ReorderableFragment, ThreadSafety};
use lignin_schema::{
	events::click,
	html::elements::{button, div},
	EventInfo,
};
use std::{
	fmt::Write,
	sync::{
		atomic::{AtomicBool, Ordering},
		Mutex,
	},
	task::Waker,
};
use tap::Pipe;
use tracing::error;
use wasm_bindgen::{JsCast, JsValue};

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

#[derive(Debug)]
pub struct Error(InnerError);

#[derive(Debug)]
enum InnerError {
	DepthLimitExceeded,
	Unsupported { message: String },
}

pub static UPDATE: AtomicBool = AtomicBool::new(false);

lazy_static::lazy_static! {
	pub static ref ON_PRESENTED: Mutex<Vec<Waker>> = Mutex::new(Vec::new());
}

fn drain_on_presented() {
	for waker in ON_PRESENTED.lock().unwrap().drain(..) {
		waker.wake()
	}
}

pub fn render<'a, S: 'static + ThreadSafety>(
	vdom: &'a Node<'a, S>,
	parent: &mut StyledDom,
	depth_limit: usize,
) -> Result<(), Error> {
	if depth_limit == 0 {
		return Err(Error(InnerError::DepthLimitExceeded));
	}

	match vdom {
		Node::Comment {
			comment: _,
			dom_binding: _,
		} => (),

		Node::HtmlElement {
			element,
			dom_binding: _,
		} => render_element(element, parent, depth_limit)?,

		Node::MathMlElement {
			element: _,
			dom_binding: _,
		} => unimplemented!(),

		Node::SvgElement {
			element: _,
			dom_binding: _,
		} => todo!("Node::SvgElement"),

		Node::Memoized {
			state_key: _,
			content,
		} => {
			// There should be some caching here.
			render(content, parent, depth_limit - 1)?
		}

		Node::Multi(multi) => {
			for node in *multi {
				render(node, parent, depth_limit - 1)?
			}
		}

		Node::Keyed(keyed) => {
			for ReorderableFragment {
				dom_key: _,
				content,
			} in *keyed
			{
				render(content, parent, depth_limit - 1)?
			}
		}

		Node::Text {
			text,
			dom_binding: _,
		} => parent.append_child(
			Dom::text(AzString::from_string((*text).to_string())).style(Css::empty()),
		),

		Node::RemnantSite(_) => unimplemented!("Node::RemnantSite"),
	}

	Ok(())
}

fn render_element<S: 'static + ThreadSafety>(
	element: &Element<S>,
	parent: &mut StyledDom,
	depth_limit: usize,
) -> Result<(), Error> {
	let Element {
		name: element_name,
		creation_options: _,
		attributes: _,
		ref content,
		event_bindings,
	} = *element;
	match element_name {
		button::TAG_NAME => {
			let mut text = String::new();
			collect_text(content, &mut text, depth_limit - 1)?;
			let mut dom_button = Button::new(AzString::from_string(text));
			for &EventBinding {
				name: event_name,
				callback,
				options: _,
			} in event_bindings
			{
				match event_name {
					<dyn click>::NAME => dom_button.set_on_click(RefAny::new(callback), {
						extern "C" fn on_click<S: 'static + ThreadSafety>(
							callback: &mut RefAny,
							_callback_info: &mut CallbackInfo,
						) -> Update {
							callback
								.downcast_ref::<CallbackRef<S, fn(event: lignin::web::Event)>>()
								.unwrap()
								.call(lignin::web::Event::new(web_sys::Event::unchecked_from_js(
									JsValue::UNDEFINED,
								)));

							if UPDATE.load(Ordering::Relaxed) {
								drain_on_presented();
								Update::RefreshDom
							} else {
								Update::DoNothing
							}
						}
						on_click::<S>
					}),
					_event => unimplemented!("Event {} on {}", event_name, element_name),
				}
			}
			dom_button.dom().style(Css::empty())
		}
		div::TAG_NAME => {
			let mut dom_div = Dom::div().style(Css::empty());
			render(content, &mut dom_div, depth_limit - 1)?;
			dom_div
		}
		name => todo!("Element with name: {}", name),
	}
	.pipe(|child| parent.append_child(child));

	Ok(())
}

fn collect_text<S: ThreadSafety>(
	node: &Node<S>,
	w: &mut impl Write,
	depth_limit: usize,
) -> Result<(), Error> {
	if depth_limit == 0 {
		error!("Depth limit exceeded.");
		return Err(Error(InnerError::DepthLimitExceeded));
	}

	match node {
		Node::Comment { .. } => (),

		Node::HtmlElement { .. } | Node::MathMlElement { .. } | Node::SvgElement { .. } => todo!(),

		Node::Memoized {
			state_key: _,
			content,
		} => collect_text(content, w, depth_limit - 1)?,

		Node::Multi(multi) => {
			for node in *multi {
				collect_text(node, w, depth_limit - 1)?
			}
		}

		Node::Keyed(keyed) => {
			for ReorderableFragment {
				dom_key: _,
				content,
			} in *keyed
			{
				collect_text(content, w, depth_limit - 1)?
			}
		}

		Node::Text {
			text,
			dom_binding: _,
		} => w.write_str(text).unwrap(),
		Node::RemnantSite(_) => unimplemented!(),
	}

	Ok(())
}
