import "solid-js";
import {createSignal} from "solid-js";

const Example = () => {
  const [count, setCount] = createSignal(0);

  return (
      <div style="display: flex; align-items: center;">
          <button onClick={() => {setCount(c => c + 1)}}>+</button>
          {count()}
          <button onClick={() => {setCount(c => c - 1)}}>-</button>
      </div>
  );
};

export default Example;
