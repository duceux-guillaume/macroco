export const REPO_URL = 'https://github.com/duceux-guillaume/macroco';

const BUG_TEMPLATE = 'bug_report.md';
const FEATURE_TEMPLATE = 'feature_request.md';
const BUG_LABEL = 'bug';
const FEATURE_LABEL = 'enhancement';

function buildIssueUrl(options: { template: string; labels: string; body?: string }): string {
	const p: Record<string, string> = { template: options.template, labels: options.labels };
	if (options.body) p.body = options.body;
	return `${REPO_URL}/issues/new?${new URLSearchParams(p).toString()}`;
}

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

	return buildIssueUrl({ template: BUG_TEMPLATE, labels: BUG_LABEL, body });
}

export function buildFeatureRequestUrl(): string {
	return buildIssueUrl({ template: FEATURE_TEMPLATE, labels: FEATURE_LABEL });
}
