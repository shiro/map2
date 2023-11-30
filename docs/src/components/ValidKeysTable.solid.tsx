import {Show, For} from "solid-js/web";

import keyEnumCode from "@project/evdev-rs/src/enums.rs?raw";
import aliasEnumCode from "@project/src/key_defs.rs?raw";


const keys = (() => {
  const code = keyEnumCode;
  const pat = "pub enum EV_KEY {";
  const fromIdx = code.indexOf(pat) + pat.length;
  const toIdx = code.indexOf("}", fromIdx);

  const snippet = code.slice(fromIdx, toIdx);

  const literals = new Set([
    "minus",
    "equal",
    "comma",
    "dot",
    "slash",
    "grave",
    "semicolon",
    "apostrophe",
    "leftbrace",
    "rightbrace",
    "backslash",
  ]);

  return snippet
    .split(",")
    .map(x => x.trim())
    .map(x => x.slice("KEY_".length, x.indexOf(" ")))
    .map(x => x.toLowerCase())
    .filter(x => x.length > 1)
    .filter(x => !literals.has(x));
})();

const aliases = (() => {
  const code = aliasEnumCode;
  const pat = "let mut m = HashMap::new();";
  const fromIdx = code.indexOf(pat) + pat.length;
  const toIdx = code.indexOf("m\n", fromIdx);

  const snippet = code.slice(fromIdx, toIdx);

  return Object.fromEntries(
    snippet
      .split(";")
      .map(x => x.trim())
      .filter(Boolean)
      .map(x => new RegExp(`"(.*)".*KEY_([^.]+)`).exec(x).slice(1, 3))
      .map(([alias, key]) => [key.toLowerCase(), alias.toLowerCase()])
    );
})();

console.log(aliases);


const descriptions = {
  down: "'down' directional key",
  f1: "function 1",
  f2: "function 2",
  f3: "function 3",
  f4: "function 4",
  f5: "function 5",
  f6: "function 6",
  f7: "function 7",
  f8: "function 8",
  f9: "function 9",
  f10: "function 10",
  f11: "function 11",
  f12: "function 12",
  f13: "function 13",
  f14: "function 14",
  f15: "function 15",
  f16: "function 16",
  f17: "function 17",
  f18: "function 18",
  f19: "function 19",
  f20: "function 20",
  f21: "function 21",
  f22: "function 22",
  f23: "function 23",
  f24: "function 24",
  kp0: "keypad 0",
  kp1: "keypad 1",
  kp2: "keypad 2",
  kp3: "keypad 3",
  kp4: "keypad 4",
  kp5: "keypad 5",
  kp6: "keypad 6",
  kp7: "keypad 7",
  kp8: "keypad 8",
  kp9: "keypad 9",
  kpasterisk: "keypad '*'",
  kpcomma: "keypad ','",
  kpdot: "keypad '.'",
  kpenter: "keypad 'center'",
  kpequal: "keypad '='",
  kpjpcomma: "keypad Japanese '、'",
  kpleftparen: "keypad '('",
  kprightparen: "keypad ')'",
  left: "'left' directional key",
  leftalt: "left meta",
  leftctrl: "left control",
  leftmeta: "left meta",
  leftshift: "left shift",
  right: "'right' directional key",
  rightalt: "right alt",
  rightctrl: "right control",
  rightmeta: "right meta",
  rightshift: "right shift",
  up: "'up' directional key",
  yen: "JPY (円)",
};



const ValidKeysTable = () => {
  return (
    <>
      <table>
      <tbody>
        <tr>
          <th>Key names</th>
          <th>Description</th>
        </tr>
        <For each={keys}>
          {(key) =>
            <tr>
              <td>
                <Show when={aliases[key]}>
                  {aliases[key]}
                  <br />
                </Show>
                {key}
              </td>
              <td>{descriptions[key]}</td>
            </tr>
          }
        </For>
      </tbody>
      </table>
    </>
  );
}

export default ValidKeysTable;
