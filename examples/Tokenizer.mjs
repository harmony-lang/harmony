import * as Console from "file:///C:/dev/rust/harmony/runtime/Console/IO.mjs";
import * as Char from "file:///C:/dev/rust/harmony/runtime/Data/Char.mjs";
import { Just, Nothing } from "file:///C:/dev/rust/harmony/runtime/Data/Maybe.mjs";
import { Success, Failure } from "file:///C:/dev/rust/harmony/runtime/Data/Result.mjs";
import * as String from "file:///C:/dev/rust/harmony/runtime/Data/String.mjs";
export const Token = {
    TkNumber: (value0) => ({
        "TkNumber0": value0,
    }),
    TkPlus: { "TkPlus": undefined },
    TkMinus: { "TkMinus": undefined },
    TkTimes: { "TkTimes": undefined },
    TkDiv: { "TkDiv": undefined },
    TkLParen: { "TkLParen": undefined },
    TkRParen: { "TkRParen": undefined },
    TkError: (value0) => ({
        "TkError0": value0,
    }),
};
export const TkNumber = Token.TkNumber;
export const TkPlus = Token.TkPlus;
export const TkMinus = Token.TkMinus;
export const TkTimes = Token.TkTimes;
export const TkDiv = Token.TkDiv;
export const TkLParen = Token.TkLParen;
export const TkRParen = Token.TkRParen;
export const TkError = Token.TkError;
export var tokenize_input = (input, index, tokens) => {
    return (() => {
    const __condition = String.length(input);
    if (__condition === 0) {
        return Just(tokens);
    }
    return (() => {
    const __condition = String.get(input, index);
    if (__condition.Just0 !== undefined) {
        const c = __condition.Just0;
        return (() => {
    const __condition = Char.isDigit(c);
    if (__condition === true) {
        return tokenize_input(input, index + 1, tokens.concat([TkNumber(Char.toInt(c))]));
    }
    if (__condition === false) {
        return (() => {
    const __condition = c;
    if (__condition === '+') {
        return tokenize_input(input, index + 1, tokens.concat([TkPlus]));
    }
    if (__condition === '-') {
        return tokenize_input(input, index + 1, tokens.concat([TkMinus]));
    }
    if (__condition === '*') {
        return tokenize_input(input, index + 1, tokens.concat([TkTimes]));
    }
    if (__condition === '/') {
        return tokenize_input(input, index + 1, tokens.concat([TkDiv]));
    }
    if (__condition === '(') {
        return tokenize_input(input, index + 1, tokens.concat([TkLParen]));
    }
    if (__condition === ')') {
        return tokenize_input(input, index + 1, tokens.concat([TkRParen]));
    }
    return tokenize_input(input, index + 1, tokens.concat([TkError("Unexpected character")]));
})();
    }
    throw new Error("Pattern match failed");
})();
    }
    if (__condition.Nothing === undefined) {
        return Just(tokens);
    }
    throw new Error("Pattern match failed");
})();
})();
}
export var tokenize = (input) => {
    return tokenize_input(input, 0, []);
}
var main = (() => {
    return (() => {
    const __condition = tokenize("1+2*3");
    if (__condition.Just0 !== undefined) {
        const tokens = __condition.Just0;
        return Console.println(tokens);
    }
    if (__condition.Nothing === undefined) {
        return Console.println("Error");
    }
    throw new Error("Pattern match failed");
})();
})();
