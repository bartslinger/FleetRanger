// @carbon/icons ships without types for its per-icon modules.
declare module '@carbon/icons/es/*' {
	const icon: import('./Icon.svelte').IconDescriptor;
	export default icon;
}
