// @carbon/pictograms ships without types for its per-pictogram modules. They
// have the same shape as icons, so the Icon component draws them.
declare module '@carbon/pictograms/es/*' {
	const pictogram: import('$ui/Icon.svelte').IconDescriptor;
	export default pictogram;
}
