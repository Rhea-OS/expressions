declare module 'expression' {
    enum TokenType {
        Name = 0,
        Operator = 1,
        LParen = 2,
        LBracket = 3,
        LBrace = 4,
        RParen = 5,
        RBracket = 6,
        RBrace = 7,
        Dot = 8,
        Comma = 9,
        Nothing = 10,
        Num = 11,
        String = 12,
        Bool = 13,
        Address = 14,
    }

    class Token {
        token(): string;
        type: TokenType
    }

    class Operator {
        constructor(token: string, handler: (...operands: any[]) => any);
    }

    class Context<Provider extends DataSource> {
        constructor(provider: Provider);
        provider(): Provider;
        clone(): Context<Provider>;

        pushGlobal(name: string, value: any): void;
        withGlobal(name: string, value: any): Context<Provider>;

        pushOperator(name: string, operator: Operator): void;
        withOperator(name: string, operator: Operator): Context<Provider>;

        evaluateStr(expr: string, cx: any): any;
        parseStr(expr: string): Token[];
    }

    interface DataSource {
        query(cx: any, query: string): any;
    }
}