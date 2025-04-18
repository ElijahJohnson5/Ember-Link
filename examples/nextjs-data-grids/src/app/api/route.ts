import * as Y from 'yjs';
import type { StorageEndpointResponse } from '@ember-link/protocol';

// This is our fake data from our DB
const data = [
  {
    name: 'Norma Fisher',
    company: 'SHEPPARD-TUCKER',
    email: 'nhoward@hotmail.com',
    phone: '(421)948-9241'
  },
  {
    name: 'Stephanie Sutton',
    company: 'CASTRO-GOMEZ',
    email: 'tammywoods@hotmail.com',
    phone: '080-160-9753x513'
  },
  {
    name: 'Connie Pratt',
    company: 'PRATTGROUP',
    email: 'kaisernancy@rodriguez-arnold.com',
    phone: '+1-583-989-4719'
  },
  {
    name: 'Emily Allen',
    company: 'LOPEZANDSONS',
    email: 'thomas12@mclean.net',
    phone: '+1-684-833-9694x77515'
  },
  {
    name: 'Katelyn Mccoy',
    company: 'SANCHEZLTD',
    email: 'thorntonnathan@gmail.com',
    phone: '601-230-9891x01399'
  },
  {
    name: 'Brandon Bass',
    company: 'JOHNSONINC',
    email: 'jennifermendoza@owen.biz',
    phone: '+1-141-314-5620x870'
  },
  {
    name: 'Melissa Myers',
    company: 'HOWARD-DENNIS',
    email: 'tammybrown@moore.com',
    phone: '+1-720-769-8456x428'
  },
  {
    name: 'Judith Miller',
    company: 'WILLIAMS,CAMPBELLANDALLEN',
    email: 'chelsea59@hotmail.com',
    phone: '+1-246-610-9352x337'
  },
  {
    name: 'Shawn Carroll',
    company: 'JOHNSON-SAVAGE',
    email: 'martincaleb@hotmail.com',
    phone: '(787)890-0754'
  },
  {
    name: 'Amber Carson',
    company: 'LOPEZ-KEY',
    email: 'valeriemorales@butler.com',
    phone: '+1-891-319-3442x17610'
  }
];

// Just always return the same data for example
// but you could use the `channel_name` sent in the request to get different data based on channel
export async function GET(_request: Request) {
  const doc = new Y.Doc();

  const yArray = doc.getArray('data');

  yArray.insert(0, data);

  const update = Y.encodeStateAsUpdate(doc);

  return Response.json({
    updates: [Array.from(update)]
  } satisfies StorageEndpointResponse);
}
