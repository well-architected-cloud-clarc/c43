import { Architecture, Entity, external } from '@arinoto/cdk-arch';

export const webArch = new Architecture('cdk-arch-web');

export const user = new Entity(external, 'user', 'person').uses(webArch, 'says hello');
