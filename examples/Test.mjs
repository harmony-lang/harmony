import * as Console from "file:///C:/dev/rust/harmony/runtime/Console/IO.mjs";
export const Maybe = {
    Just: (value0) => ({
        "Just0": value0,
    }),
    Nothing: { "Nothing": undefined },
};
export const Just = Maybe.Just;
export const Nothing = Maybe.Nothing;
export function test(x) {
    return (() => {
        const __condition = x;
        if (__condition.Just0 !== undefined) {
            const x = __condition.Just0;
            return Console.println(x);
        }
        if (__condition.Nothing === undefined) {
            return Console.println("Nothing");
        }
        throw new Error("Pattern match failed");
    })();
}
export default function main() {
    return test(Nothing);
}
main();