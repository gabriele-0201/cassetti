use pipewire::{
    link::Link, prelude::ReadableDict, properties, types::ObjectType, Context, MainLoop,
};
use std::cell::RefCell;
use std::rc::Rc;

pub fn pipewire_test() {
    let mainloop = MainLoop::new().unwrap();
    let context = Context::new(&mainloop).unwrap();
    let core = Rc::new(context.connect(None).unwrap());
    let registry = Rc::new(Rc::clone(&core).get_registry().unwrap());
    let registry_2 = Rc::clone(&registry);

    // What I need to save here?
    //
    // I need a way to store the node and port id of CALF stuff

    // all of those are IDs
    let calf_node: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let calf_port_in: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    //let calf_port_inr: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let calf_port_out: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    //let calf_port_outr: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));

    let playback_node: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    //let node_playback_r: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let playback_port: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    //let port_playback_r: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));

    let cassette_node: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    //let node_playback_r: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    let cassette_port: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));
    //let port_playback_r: Rc<RefCell<Option<u32>>> = Rc::new(RefCell::new(None));

    let new_link = Rc::new(
        move |calf_node: &Rc<RefCell<Option<u32>>>,
              calf_port_in: &Rc<RefCell<Option<u32>>>,
              node_from: u32,
              port_from: u32| {
            let calf_node = Rc::clone(calf_node)
                .borrow()
                .expect("Impossible create link without CLAF");

            let calf_port_in = Rc::clone(calf_port_in)
                .borrow()
                .expect("Impossible create link without CLAF");

            Rc::clone(&core).create_object::<Link, _>(
                "link-factory",
                &properties! {
                    "link.output.node" => node_from.to_string(),
                    "link.output.port" => port_from.to_string(),
                    "link.input.node" => calf_node.to_string(),
                    "link.input.port" => calf_port_in.to_string(),
                    "object.linger" => "1"
                },
            );
        },
    );

    let _listener = registry
        .add_listener_local()
        .global(move |global| {
            match &global.type_ {
                ObjectType::Link => ()/*match &global.props {
                    Some(props) => match props.get("factory.id") {
                        Some("20") => {
                            // If this is a link THAN I have to test if this is the link
                            // between player and playback that I have to remove

                            // All the nodes and ports ids needs to be defined
                            match (
                                *Rc::clone(&playback_node).borrow(),
                                *Rc::clone(&playback_port).borrow(),
                                *Rc::clone(&cassette_node).borrow(),
                                *Rc::clone(&cassette_port).borrow(),
                            ) {
                                (Some(pn), Some(pp), Some(cn), Some(cp)) => {
                                    // Delete the the link if it is the correct one

                                    println!("playback node:{}, playback port:{}, cass node:{}, cass port:{}", pn, pp, cn, cp);
                                    println!("{props:?}");

                                    if let (
                                        Some(cp_str),
                                        Some(pp_str),
                                        Some(cn_str),
                                        Some(pn_str),
                                    ) = (
                                        props.get("link.output.port"),
                                        props.get("link.input.port"),
                                        props.get("link.output.node"),
                                        props.get("link.input.node"),
                                    ) {
                                        if (pn, pp, cn, cp)
                                            == (
                                                pn_str.parse().unwrap(),
                                                pp_str.parse().unwrap(),
                                                cn_str.parse().unwrap(),
                                                cp_str.parse().unwrap(),
                                            )
                                        {
                                            println!("destroyed");
                                            registry_2.destroy_global(global.id);

                                            // NOW I should be able to create a link between this and the
                                            println!("created");
                                            Rc::clone(&new_link)(&calf_node, &calf_port_in, cn, cp);
                                        }
                                    }

                                }
                                _ => (),
                            }
                        }
                        _ => (),
                    },
                    _ => (),

                }*/,
                ObjectType::Port => {
                    match &global.props {
                        Some(props) => match (
                            props.get("object.path"),
                            props.get("port.name"),
                            props.get("port.id"),
                            props.get("node.id"),
                        ) {
                            (Some(_path), Some(name), Some(port_id), Some(node_id))
                                if name == "Tape Simulator In #1" =>
                            {
                                *Rc::clone(&calf_node).borrow_mut() =
                                    Some(node_id.parse().unwrap());
                                *Rc::clone(&calf_port_in).borrow_mut() =
                                    Some(port_id.parse().unwrap());
                            }
                            (Some(_path), Some(name), Some(port_id), Some(node_id))
                                if name == "Tape Simulator Out #1" =>
                            {
                                // I do it twice just because I'm lazy
                                *Rc::clone(&calf_node).borrow_mut() =
                                    Some(node_id.parse().unwrap());
                                *Rc::clone(&calf_port_out).borrow_mut() =
                                    Some(port_id.parse().unwrap());
                            }
                            (Some(path), Some(_name), Some(port_id), Some(node_id))
                                if path == "alsa_playback.Cassette:output_0" =>
                            {
                                // here I just save the information about the Cassette player
                                // and when I will destroy the link with the Playback THEN i will create the
                                // new link with calf
                                *Rc::clone(&cassette_node).borrow_mut() =
                                    Some(node_id.parse().unwrap());
                                *Rc::clone(&cassette_port).borrow_mut() =
                                    Some(port_id.parse().unwrap());

                                println!("created");
                                Rc::clone(&new_link)(&calf_node, &calf_port_in, node_id.parse().unwrap(), port_id.parse().unwrap());
                            }
                            /* let's work only in mono for now
                            (Some(path), Some(name), Some(port_id), Some(node_id))
                                if path == "alsa_playback.Cassette:output_1" =>
                            {
                                println!("{global:?}");
                            }
                            */
                            (Some(path), Some(_name), Some(port_id), Some(node_id))
                                if path == "alsa:pcm:1:front:1:playback:playback_0" =>
                            {
                                println!(" cassette: {props:?}");
                                *Rc::clone(&playback_node).borrow_mut() =
                                    Some(node_id.parse().unwrap());
                                *Rc::clone(&playback_port).borrow_mut() =
                                    Some(port_id.parse().unwrap());
                            }
                            _ => (),
                        },
                        None => (),
                    };
                }
                _ => (),
            }
        })
        .global_remove(|id_removed| {
            println!("Removed: {id_removed:?}");
        })
        .register();

    mainloop.run();
}

/*
 * * Example dict
 *
 * ForeignDict {
*       flags: (empty),
*       entries: {
*           "object.serial": "883",
*           "object.path": "Calf Studio Gear:input_2",
*           "format.dsp": "32 bit float mono audio",
*           "node.id": "59",
*           "port.id": "2",
*           "port.name": "Tape Simulator In #1",
*           "port.direction": "in",
*           "port.alias": "Calf Studio Gear:Tape Simulator In #1"
*      }
* }
*
GlobalObject { id: 35,
    permissions: R | W | X | M,
    type_: Port,
    version: 3,
    props: Some(ForeignDict { flags: (empty),
    entries: {
    "object.serial": "54",
    "object.path": "alsa:pcm:1:front:1:playback:playback_0",
    "format.dsp": "32 bit float mono audio",
    "node.id": "52",
    "audio.channel": "FL",
    "port.id": "0",
    "port.name": "playback_FL",
    "port.direction": "in",
    "port.physical": "true",
    "port.terminal": "true",
    "port.alias": "ALC257 Analog:playback_FL"
} }) }

link example:
GlobalObject {
id: 63,
permissions: R | W | X | M,
type_: Link,
version: 3,
props: Some(
ForeignDict { flags: (empty),
entries: {
"object.serial": "880",
"factory.id": "20",
"link.output.port": "56",
"link.input.port": "35",
"link.output.node": "66",
"link.input.node": "52"} }) }
*
*/
