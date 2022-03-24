//! TODO_DOCS_DESCRIPTION
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Flignin-azul)](https://iteration-square.schichler.dev/#narrow/stream/project.2Flignin-azul)

#![doc(html_root_url = "https://docs.rs/lignin-azul/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(missing_docs, clippy::missing_errors_doc, clippy::missing_panics_doc)] // TODO

use std::fmt::Write;

use azul::{
	dom::Dom,
	prelude::{Css, StyledDom},
	str::String as AzString,
	widgets::Button,
};
use lignin::{Element, Node, ReorderableFragment, ThreadSafety};
use tap::Pipe;
use tracing::error;

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

pub fn render<'a, S: ThreadSafety>(
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

fn render_element<S: ThreadSafety>(
	element: &Element<S>,
	parent: &mut StyledDom,
	depth_limit: usize,
) -> Result<(), Error> {
	let Element {
		name,
		creation_options,
		attributes,
		ref content,
		event_bindings,
	} = *element;
	match name {
		lignin_schema::html::elements::button::TAG_NAME => {
			let mut text = String::new();
			collect_text(content, &mut text, depth_limit - 1)?;
			let mut button = Button::new(AzString::from_string(text));
			button.dom().style(Css::empty())
		}
		lignin_schema::html::elements::div::TAG_NAME => {
			let mut div = Dom::div().style(Css::empty());
			render(content, &mut div, depth_limit - 1)?;
			div
		}
		name => todo!("Element with name: {}", name),
	}
	.pipe(|child| parent.append_child(child));

	Ok(())
}

fn collect_text<S: ThreadSafety>(
	content: &Node<S>,
	w: &mut impl Write,
	depth_limit: usize,
) -> Result<(), Error> {
	if depth_limit == 0 {
		error!("Depth limit exceeded.");
		return Err(Error(InnerError::DepthLimitExceeded));
	}
	todo!()
}
