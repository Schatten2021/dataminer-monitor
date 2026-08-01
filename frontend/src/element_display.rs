use yew::Html;
use api_types::AttributeValue;

#[derive(Debug, Properties, PartialEq)]
pub struct Props {
    pub element: crate::status::Element,
    pub id: String,
}

#[function_component(ElementDisplay)]
pub fn element_display(props: &Props) -> Html {
    html!{
        <div class={if props.element.online {"element element-online"} else { "element element-offline" }}>
            <h2><b class={if props.element.online { "status-online status" } else { "status-offline status" }}>{"⬤"}</b>{"   "}{&props.id}</h2>
            <div class="attributes">{
                props.element.attributes.iter()
                    .map(|(a, b)| (a.clone(), b.clone()))
                    .map(|(id, val)| html!(<AttributeDisplay id={id} value={val}/>))
                    .collect::<Html>()
            }</div>
        </div>
    }
}
#[derive(Debug, Properties, PartialEq)]
pub struct AttributeDisplayProps {
    id: String,
    value: AttributeValue,
}
#[function_component(AttributeDisplay)]
fn display_attribute(props: &AttributeDisplayProps) -> Html {
    let rendered_value = render_attr_value(&props.value);
    html!{
        <div class="attr">{&props.id}{": "}{rendered_value}</div>
    }
}
fn render_attr_value(value: &AttributeValue) -> String {
    match value {
        AttributeValue::Unit => "()".to_string(),
        AttributeValue::Boolean(v) => format!("{v}"),
        AttributeValue::Count(v) => format!("{v}"),
        AttributeValue::Date(d) => d.naive_local().format("%d.%m.%Y %H:%M:%S%.3f").to_string(),
        AttributeValue::Percentage(v) => format!("{:.2}", v * 100.0),
        AttributeValue::List(values) => format!("[{}]", values.iter()
            .map(render_attr_value)
            .collect::<Vec<_>>()
            .join(", ")),
        AttributeValue::Number(v) => format!("{v}"),
        AttributeValue::String(v) => v.clone(),
        AttributeValue::Enum(v) => format!("{}({})", v.variant, render_attr_value(&v.value)),
        AttributeValue::Map(map) => format!("{{{}}}", map.iter()
            .map(|(k, v)| format!("{}: {}", render_attr_value(k), render_attr_value(v)))
            .collect::<Vec<_>>()
            .join(", ")
        )
    }
}