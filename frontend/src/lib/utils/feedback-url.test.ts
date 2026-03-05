// REQ: REQ-028
import { describe, it, expect } from 'vitest';
import { buildBugReportUrl, buildFeatureRequestUrl, REPO_URL } from './feedback-url';

describe('feedback URL builders', () => {
	it('builds bug report URL with encoded body', () => {
		const url = buildBugReportUrl('Collapse', 'Mozilla/5.0 Test');
		expect(url).toContain(`${REPO_URL}/issues/new`);
		expect(url).toContain('template=bug_report.md');
		expect(url).toContain('labels=bug');
		const bodyParam = new URL(url).searchParams.get('body');
		expect(bodyParam).toContain('Collapse');
		expect(bodyParam).toContain('Mozilla/5.0 Test');
	});

	it('builds feature request URL', () => {
		const url = buildFeatureRequestUrl();
		expect(url).toContain(`${REPO_URL}/issues/new`);
		expect(url).toContain('template=feature_request.md');
		expect(url).toContain('labels=enhancement');
	});

	it('handles null preset gracefully', () => {
		const url = buildBugReportUrl(null, 'Agent');
		const bodyParam = new URL(url).searchParams.get('body');
		expect(bodyParam).toContain('None');
	});
});
