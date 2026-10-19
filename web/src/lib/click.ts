/** A plain click on a link, not one meant to open it elsewhere (a new tab or window). */
export function plainClick(event: MouseEvent): boolean {
	return event.button === 0 && !event.metaKey && !event.ctrlKey && !event.shiftKey && !event.altKey;
}
