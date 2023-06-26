import * as Console from "file:///C:/dev/rust/harmony/runtime/Console/IO.mjs";
import * as Char from "file:///C:/dev/rust/harmony/runtime/Data/Char.mjs";
import { Just, Nothing } from "file:///C:/dev/rust/harmony/runtime/Data/Maybe.mjs";
import { Success, Failure } from "file:///C:/dev/rust/harmony/runtime/Data/Result.mjs";
import * as String from "file:///C:/dev/rust/harmony/runtime/Data/String.mjs";

export const Token = {
    TkNumber: (value0) => ({
        "TkNumber0": value0
    }),
    TkPlus: () => ({}),
    TkMinus: () => ({}),
    TkTimes: () => ({}),
    TkDiv: () => ({}),
    TkLParen: () => ({}),
    TkRParen: () => ({}),
    TkError: (value0) => ({
        "TkError0": value0
    })
};
export function tokenize_input(input, index, tokens) {
    return (() => {
        const __condition = String.length(input);
        if (__condition === 0) {
            return Success.create(tokens);
        }
        return (() => {
            const __condition = String.get(input, index);
            if (__condition instanceof Just) {
                const c = __condition.value0;
                return (() => {
                    const __condition = Char.isDigit(c);
                    if (__condition === true) {
                        return tokenize_input(input, index + 1, tokens + [Token.TkNumber(Char.toInt(c))]);
                    }
                    if (__condition === false) {
                        return (() => {
                            const __condition = c;
                            if (__condition === '+') {
                                return tokenize_input(input, index + 1, tokens + [Token.TkPlus()]);
                            }
                            if (__condition === '-') {
                                return tokenize_input(input, index + 1, tokens + [Token.TkMinus()]);
                            }
                            if (__condition === '*') {
                                return tokenize_input(input, index + 1, tokens + [Token.TkTimes()]);
                            }
                            if (__condition === '/') {
                                return tokenize_input(input, index + 1, tokens + [Token.TkDiv()]);
                            }
                            if (__condition === '(') {
                                return tokenize_input(input, index + 1, tokens + [Token.TkLParen()]);
                            }
                            if (__condition === ')') {
                                return tokenize_input(input, index + 1, tokens + [Token.TkRParen()]);
                            }
                            return tokenize_input(input, index + 1, tokens + [Token.TkError("Unexpected character")]);
                        })();
                    }
                    // won't get generated because the compiler knows
                    // that all cases are covered
                    // throw new Error("Pattern match failed");
                })();
            }
            if (__condition instanceof Nothing) {
                return tokenize_input(input, index + 1, tokens + [TkError.create("Unexpected end of input")]);
            }
            // won't get generated because the compiler knows
            // that all cases are covered
            // throw new Error("Pattern match failed");
        })();
    })();
}
export function tokenize(input) {
    return tokenize_input(input, 0, []);
}
export default function main() {
    return (() => {
        const __condition = tokenize("1+2*3");
        if (__condition instanceof Success) {
            const tokens = __condition.value0;
            console.log(tokens);
            return Console.println(tokens);
        }
        if (__condition instanceof Failure) {
            const error = __condition.value0;
            console.log(error);
            return Console.println(error);
        }
        // won't get generated because the compiler knows
        // that all cases are covered
        // throw new Error("Pattern match failed");
    })();
}

main();
