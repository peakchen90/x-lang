import * as path from 'path';
import * as fs from 'fs';
import * as xlang from '../src/index';

describe("parse", () => {
    test("version", () => {
        expect(xlang.version).toBe(require('../../package.json').version);
    });

    test("fixtures", () => {
        const fixturesDir = path.join(__dirname, './fixtures');
        fs.readdirSync(fixturesDir).forEach(name => {
            const match = name.match(/^(.+)\.x$/);
            if (match) {
                const code = fs.readFileSync(path.join(fixturesDir, name), {encoding: 'utf-8'});
                const result = JSON.parse(fs.readFileSync(
                    path.join(fixturesDir, `${match[1]}.json`), {encoding: 'utf-8'}
                ));
                expect(xlang.parse(code)).toEqual(result);
            }
        });
    })
})
