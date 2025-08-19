import { expect, test } from 'vitest'
import { Dggrs1 } from '../src'

test('myFunction', () => {
     const g = new Dggrs1("isea3h");
     const bbox = [
       [-10.0, -10.0],
       [10.0, -10.0],
       [10.0, 10.0],
       [-10.0, 10.0],
     ];
     const a = g.zonesFromBbox1(1, false, bbox);
  // expect(myFunction()).toBe('Hello, world!')
})
