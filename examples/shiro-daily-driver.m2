print("start");

// kill script key
!{pagedown}::{ exit(); };

let key_pressed = false;
let tab_pressed = false;

{tab down}::{
  tab_pressed = true;
  key_pressed = false;
};


{tab up}::{
  tab_pressed = false;
  if(key_pressed == false){
    send("{tab}");
  }
};


let key_tab_mod = |key|{
  if(tab_pressed){
    send( "{alt down}{meta down}{shift down}" + key + "{alt up}{meta up}{shift up}");
  }
};


let caps_down = false;

^capslock::capslock;

{capslock down}::{
  caps_down = true;
  key_pressed = false;
  send("{ctrl down}");
};

{capslock up}::{
  caps_down = false;
  send("{ctrl up}");
  if (key_pressed == false){
    send("{esc}");
  }
};



let lalt = false;
{leftalt down}::{lalt = true; send("{leftalt down}");};
!{leftalt up}::{lalt = false; send("{leftalt up}");};

let ralt = false;
{rightalt down}::{ralt = true; send("{rightalt down}");};
!{rightalt up}::{ralt = false; send("{rightalt up}");};


let directional_mod = |key, direction|{
  let map = |key_down, key_down_str, key_up, key_up_str|{
    map_key(key_down, ||{
      key_pressed = true;

      if (lalt){ send("{"+direction+" down}"); return 0; }
      if (caps_down){
        send("{alt down}{meta down}{shift down}{ctrl down}"+key+"{alt up}{meta up}{shift up}{ctrl up}");
        return 0;
      }
      if (ralt){ send("{rightalt up}{alt down}{meta down}"+key+"{alt up}{meta up}{rightalt down}"); return 0; }

      if (key_tab_mod(key_down)){ return 0; }
      send(key_down_str);
    });

    map_key(key_up, ||{
      if (lalt){ send("{"+direction+" up}"); return 0; }
      if (ralt){ return 0; }
      send(key_up_str);
    });
  };

  let key_down = "{"+key+" down}";
  let key_up = "{"+key+" up}";

  map(key_down, key_down ,key_up, key_up);
  map("!"+key_down, "{alt down}"+key_down+"{alt up}", "!"+key_up, "{alt down}"+key_up+"{alt up}");
};

let handle_key = |key|{
  let key_down = "{"+key+" down}";
  let key_up = "{"+key+" up}";

  if(key == "h"){ directional_mod(key, "left"); return 0; }
  if(key == "j"){ directional_mod(key, "down"); return 0; }
  if(key == "k"){ directional_mod(key, "up"); return 0; }
  if(key == "l"){ directional_mod(key, "right"); return 0; }

  map_key(key_down, ||{
    key_pressed = true;
    if (key_tab_mod(key_down)){ return 0; }
    send(key_down);
  });
};

for(let i=97; i<97+26; i=i+1){ handle_key(number_to_char(i)); }
for(let i=char_to_number("0"); i<char_to_number("9"); i=i+1){ handle_key(number_to_char(i)); }
handle_key("space");
handle_key("/");
handle_key(";");


let setup_mouse = ||{
  f13::kp1;
  f14::kp2;
  f15::kp3;
  f16::kp4;
  f17::kp5;
  f18::kp6;
  f19::kp7;
  f20::kp8;
  f21::kp9;
};

setup_mouse();


let map_figma_shortcut = |key, command|{
  map_key(key, ||{
    send("{ctrl down}/{ctrl up}");
    sleep(200);
    send(command+"{enter}");
  });
};

on_window_change(||{
  setup_mouse();

  if(active_window_class() == "firefox"){
    f13::^tab;
    +f13::+^tab;
    f14::^t;
    f16::f5;
    f21::^w;
  }else if(active_window_class() == "figma-linux"){
    map_figma_shortcut("f13", "palette-pick");
    map_figma_shortcut("f14", "atom-sync");
    map_figma_shortcut("f15", "batch styler");
    map_figma_shortcut("f16", "chroma colors");
    map_figma_shortcut("f17", "scripter");
    map_figma_shortcut("f20", "theme-flip");
  }
});
