export const REPO_URL = 'https://github.com/duceux-guillaume/macroco';

export function buildBugReportUrl(presetName: string | null, userAgent: string): string {
	const body = [
		'## Environment',
		'',
		`- **Browser / OS:** ${userAgent}`,
		`- **Active Preset:** ${presetName ?? 'None'}`,
		'',
		'## Description',
		'',
		'',
		'## Steps to Reproduce',
		'',
		'1. ',
		'',
		'## Expected Behavior',
		'',
		'',
		'## Actual Behavior',
		''
	].join('\n');

	const params = new URLSearchParams({
		template: 'bug_report.md',
		labels: 'bug',
		body
	});
	return `${REPO_URL}/issues/new?${params.toString()}`;
}

export function buildFeatureRequestUrl(): string {
	const params = new URLSearchParams({
		template: 'feature_request.md',
		labels: 'enhancement'
	});
	return `${REPO_URL}/issues/new?${params.toString()}`;
}
