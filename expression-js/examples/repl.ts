import * as rl from 'node:readline/promises';
import * as expr from 'expression';

class Table implements expr.DataSource {
    public readonly columns: string[] = [];
    private data: any[] = [];

    constructor(columns: string[] = []) {
        this.columns = columns;
    }

    query(cx: any, query: string) {
        const [, col, row] = query.match(/^(.+):(\d+)$/) ?? [];
        return this.data[Number(row) * this.columns.length + this.columns.indexOf(col)] ?? null;
    }

    set(address: string, value: any) {
        const [, col, row] = address.match(/^(.+):(\d+)$/) ?? [];
        this.data[Number(row) * this.columns.length + this.columns.indexOf(col)] = value;
    }
}

const cx = new expr.Context(new Table(["a", "b", "c"]));
const prompt = rl.createInterface(process.stdin, process.stderr);

const commands = {
    async set(address: string) {
        const body = await prompt.question('--> ');
        cx.provider().set(address, cx.evaluateStr(body, {}));
    },

    async func(args: string) {
        const [name, ...params] = args.split(/\s+/);
        const body = await prompt.question('--> ');
        cx.pushGlobal(name, function(args: any[]) {
            const inner = cx.clone();

            for (const [a, arg] of params.entries())
                inner.pushGlobal(arg, args[a]);

            return inner.evaluateStr(body, {});
        });
    },

    async glob(name) {
        const body = await prompt.question('--> ');
        cx.pushGlobal(name, cx.evaluateStr(body, {}));
    }
} satisfies Record<string, (arg: string) => Promise<void>>;

while (true) {
    const cmd = await prompt.question('> ').then(res => res.trim());

    if (cmd === '')
        continue;
    else if (cmd === '/exit')
        break;
    else if (cmd === '/dump')
        console.log(cx.provider());
    else if (cmd.startsWith('/set '))
        await commands.set(cmd.slice(5).trim());
    else if (cmd.startsWith('/func '))
        await commands.func(cmd.slice(6).trim());
    else if (cmd.startsWith('/glob '))
        await commands.glob(cmd.slice(6).trim());
    else
        console.log(cx.evaluateStr(cmd, {}));
}