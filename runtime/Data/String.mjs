import { Just, Nothing } from "file:///C:/dev/rust/harmony/runtime/Data/Maybe.mjs";
export var length = (arg0) => {
    return arg0.length;
}
export var get = (s, i) => {
    return i < 0 || i >= length(s) ? Nothing : Just(s[i]);
}
