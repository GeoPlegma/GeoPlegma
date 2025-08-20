import { expect, test } from 'vitest'
import { Dggrs } from '../src'

test('myFunction', () => {
     const g = new Dggrs("isea3h");
     const bbox = [
       [-10.0, -10.0],
       [10.0, -10.0],
       [10.0, 10.0],
       [-10.0, 10.0],
     ];
     const a = g.zonesFromBbox(1, false, bbox);
     const b = g.zoneFromPoint(1, [39, 9], false);
    //  const c = g.zonesFromParent(1, "010000000000000000", false);
     const d = g.zoneFromId("010000000000000000", false);
     console.log(a.map((v) => v.id));
     console.log(b.map(v => v.id))
     console.log(d.map((v) => v.id));
  // expect(myFunction()).toBe('Hello, world!')
})
