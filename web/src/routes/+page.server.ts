import { get, type FlightPage } from '#lib/server/backend.js';

export async function load({ fetch }) {
	return await get<FlightPage>(fetch, '/api/flights');
}
