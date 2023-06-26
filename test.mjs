export const Maybe = {
    Just: (value0) => ({
        "Just0": value0
    }),
    Nothing: { "Nothing": undefined },
};
export const Just = Maybe.Just;
export const Nothing = Maybe.Nothing;

export default function main(cond) {
    const __condition = cond;
    if (__condition.Just0 !== undefined) {
        const x = __condition.Just0;
        return x;
    }
    if (__condition.Nothing === undefined) {
        return 0;
    }
    throw new Error("Pattern match failed");
}

console.log(main(Just(1)));
console.log(main(Nothing));