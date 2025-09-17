# Gainsight API Workflows Guide

## Overview
This guide provides comprehensive workflows for interacting with Gainsight's customer success platform APIs, based on HAR analysis and reverse engineering of the web application.

## Prerequisites & Authentication

### Required Headers
```http
authtoken: eyJlbmMiOiJBMTI4R0NNIiwiYWxnIjoiZGlyIn0...
Cookie: sid=session_token; platform_sid=platform_session_token
Content-Type: application/json
x-gs-host: GAINSIGHT
User-Agent: Mozilla/5.0 (compatible client)
```

### Base URLs
- **Primary API**: `https://postman.us2.gainsightcloud.com`
- **Static Assets**: `https://staticjs.us2.gainsightapp.net`

### Authentication Token Structure
- **authtoken**: JWE encrypted token (A128GCM algorithm)
- **sid**: Session identifier (encrypted, ~4000+ chars)
- **platform_sid**: Platform session (encrypted, ~1000 chars)

## Workflow 1: Find Customer ID by Name/Partial Name

This is a 3-step process: Search Companies → Search Relationships → Resolve Assignment

### Step 1: Company Search (Partial Name Support)
```http
POST /v1/data/objects/company/search?formatDate=true
Content-Type: application/json
authtoken: {token}
Cookie: sid={session}; platform_sid={platform_session}

{
  "advanceFilters": null,
  "filters": [
    {
      "fields": ["Name"],
      "operator": "CONTAINS",
      "value": "7-11"
    }
  ],
  "includeFields": ["Name"],
  "from": 0,
  "size": 20
}
```

**Response Schema:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "requestId": "bf1818f6-4b38-49b4-a0d8-ac9cb1f5ff99",
  "data": {
    "records": [
      {
        "Name": "7 - Eleven (7 - 11) Inc.",
        "Id": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR"
      }
    ],
    "totalCount": 1
  }
}
```

### Step 2: Relationship Search (Get Relationship ID)
```http
POST /v1/data/objects/relationship/search?formatDate=true
Content-Type: application/json
authtoken: {token}
Cookie: sid={session}; platform_sid={platform_session}

{
  "advanceFilters": null,
  "filters": [
    {
      "fields": [
        "CompanyId__gr.Name",
        "Name"
      ],
      "operator": "CONTAINS",
      "value": "7-11"
    }
  ],
  "includeFields": [
    "Name",
    "CompanyId__gr.Name",
    "Postman_Team_ID__gc",
    "Csm__gr.Name"
  ],
  "from": 0,
  "size": 20
}
```

**Response Schema:**
```json
{
  "result": true,
  "requestId": "ab318e06-fd2f-4544-beea-6f65451710df",
  "data": {
    "records": [
      {
        "Name": "7 - Eleven (7 - 11) Inc.",
        "CompanyId__gr.Name": "7 - Eleven (7 - 11) Inc.",
        "Postman_Team_ID__gc": "68639",
        "Csm__gr.Name": "Jared Boynton",
        "Id": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU"
      }
    ],
    "totalCount": 1
  }
}
```

### Step 3: Assignment Resolution (Get Company Context)
```http
POST /v2/galaxy/assignment/resolve/cid
Content-Type: application/json
authtoken: {token}
Cookie: sid={session}; platform_sid={platform_session}

{
  "companyId": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
  "entityId": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
  "entityType": "company",
  "sharingType": "internal"
}
```

**Response Schema:**
```json
{
  "result": true,
  "data": {
    "layoutResolverDTO": {
      "accountId": "0011K00002H9qNmQAJ",
      "companyId": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
      "entityType": "Company",
      "sharingType": "Internal",
      "userId": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA"
    },
    "companyDetails": {
      "name": "7 - Eleven (7 - 11) Inc.",
      "industry": "Retail",
      "tier": "Enterprise"
    }
  }
}
```

### Available Search Operators:
- `CONTAINS` - Partial match (recommended for company names)
- `EQ` - Exact match
- `STARTS_WITH` - Begins with
- `ENDS_WITH` - Ends with
- `IN` - List match

### Progressive Search Strategy:
The HAR analysis shows a progressive refinement approach:
1. Search with "7" (broad)
2. Refine to "7-" (narrower)
3. Final search with "7-11" (specific)

This allows for typeahead-style functionality where results narrow as the user types.

## Workflow 2: View Report Data

### Step 1: Get Report Configuration
```http
GET /v2/galaxy/bootstrap/consumption/config/R360
authtoken: {token}
Cookie: sid={session}; platform_sid={platform_session}
```

**Response Schema:**
```json
{
  "result": true,
  "data": {
    "availableReports": [
      {
        "reportId": "83b5e7f7-87de-47f3-bb94-218ddf041dbb",
        "reportName": "API Creates WoW",
        "category": "Product Usage"
      }
    ],
    "permissions": ["READ", "WRITE"]
  }
}
```

### Step 2: Get Report Data
```http
POST /v3/bi/reporting/section/report-data?useCache=false&requestSource=R360&includeId=true&piedc=true&entityId=rId&c360RevampEnabled=true
Content-Type: application/json

{
  "reportId": "83b5e7f7-87de-47f3-bb94-218ddf041dbb",
  "userId": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
  "widgetId": "g_5958b914-3286-410f-8111-c99adb0cf6fb",
  "sectionId": "84441ebc-079b-4e3d-85f0-d0010b5a3eed",
  "widgetGlobalFilters": {
    "conditions": [{
      "leftOperand": {
        "type": "BASE_FIELD",
        "fieldName": "Postman_Team_ID__gc",
        "dataType": "LOOKUP",
        "objectName": "product_usage_team_metrics_weekly__gc"
      },
      "filterAlias": "bm_0",
      "logicalOperator": "AND",
      "comparisonOperator": "EQ",
      "rightOperandType": "VALUE",
      "filterValue": {"value": ["68639"]}
    }],
    "expression": "bm_0"
  }
}
```

**Response Schema:**
```json
{
  "result": true,
  "requestId": "uuid",
  "data": {
    "data": {
      "configuration": {
        "chart": {
          "type": "line"
        },
        "xAxis": {
          "categories": ["1/1/2024", "1/8/2024", "2/5/2024"]
        },
        "series": [{
          "name": "API creates",
          "data": [
            {
              "x": 0,
              "y": 1.0,
              "name": "1/1/2024",
              "label": "1"
            }
          ],
          "color": "rgba(65, 177, 238, 1)"
        }]
      }
    },
    "reportDefinition": {
      "reportName": "API Creates WoW",
      "sourceDetails": {
        "objectName": "product_usage_team_metrics_weekly__gc",
        "connectionType": "MDA"
      }
    }
  }
}
```

### Step 3: Get Data Count (Optional)
```http
POST /v3/bi/query/fetch-data-count?connectionType=MDA
Content-Type: application/json

{
  "reportId": "83b5e7f7-87de-47f3-bb94-218ddf041dbb",
  "filters": []
}
```

## Workflow 3: View Timeline Entries

### Get Timeline Activities
```http
POST /v1/ant/timeline/search/gsactivities?page=0&size=5
Content-Type: application/json
authtoken: {token}
Cookie: sid={session}; platform_sid={platform_session}

{
  "searchText": "",
  "quickSearch": {},
  "contextFilter": {
    "cids": [
      "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR"
    ]
  },
  "filterContext": null
}
```

**Response Schema:**
```json
{
  "result": true,
  "data": {
    "activities": [
      {
        "id": "1I004SG7RDV06L1HFJ7H61L6DIHV8OPV25TE",
        "createdDate": 1758013256980,
        "lastModifiedDate": 1758013345719,
        "note": {
          "type": "UPDATE",
          "subject": "Update",
          "content": "<p>@Sean Reed do we have a followup scheduled?</p>",
          "plainText": "@Sean Reed do we have a followup scheduled?",
          "activityDate": 1758013200000
        },
        "author": {
          "id": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
          "name": "Jared Boynton",
          "email": "jared.boynton@postman.com"
        },
        "contexts": [{
          "id": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
          "obj": "Company",
          "lbl": "7 - Eleven (7 - 11) Inc."
        }],
        "mentions": [{
          "id": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
          "name": "Sean Reed",
          "email": "sean.reed@postman.com",
          "notified": false
        }],
        "status": "POSTED",
        "attachments": []
      }
    ],
    "totalCount": 25,
    "hasMore": true
  }
}
```

### Get User Permissions for Timeline
```http
GET /v1/ant/v2/activity/user/permissions
authtoken: {token}
Cookie: sid={session}
```

**Response Schema:**
```json
{
  "result": true,
  "data": {
    "canCreate": true,
    "canEdit": true,
    "canDelete": false,
    "canMention": true,
    "availableTypes": ["UPDATE", "MEETING", "EMAIL", "CALL", "TASK"]
  }
}
```

## Workflow 4: Post to Timeline

### Create Timeline Entry
```http
POST /v1/ant/v2/activity
Content-Type: application/json

{
  "id": null,
  "createdBy": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
  "createdDate": null,
  "lastModifiedBy": null,
  "lastModifiedDate": null,
  "note": {
    "customFields": {
      "Ant__Meeting_Category__c": null,
      "externalAttendees": [],
      "internalAttendees": []
    },
    "type": "UPDATE",
    "subject": "Update",
    "activityDate": 1758013200000,
    "content": "<p><span data-profilepicture=\"null\" data-value=\"Sean Reed\" data-email=\"sean.reed@postman.com\" data-name=\"Sean Reed\" aria-label=\"Sean Reed\" title=\"Sean Reed\" data-reference=\"1758013213559\" class=\"medium-editor-at-mention\" data-id=\"1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F\"><span data-id=\"\">@Sean Reed</span></span><span data-id=\"\">&nbsp;do we have a followup scheduled with these guys?</span></p>",
    "plainText": "@Sean Reed do we have a followup scheduled with these guys?",
    "trackers": null
  },
  "context": null,
  "contexts": [{
    "id": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
    "base": true,
    "obj": "Company",
    "lbl": "7 - Eleven (7 - 11) Inc.",
    "eid": null,
    "eobj": "Account",
    "eurl": null,
    "esys": "SALESFORCE",
    "dsp": true
  }],
  "author": {
    "userId": null,
    "userName": null,
    "userEmail": null,
    "profile": null,
    "userType": null,
    "systemType": null,
    "id": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
    "name": "Jared Boynton",
    "email": "jared.boynton@postman.com",
    "companyId": null,
    "licenseType": null,
    "createdDate": null,
    "activeUser": false,
    "obj": "User",
    "eid": null,
    "eobj": "User",
    "eurl": null,
    "esys": "SALESFORCE"
  },
  "status": "POSTED",
  "tags": null,
  "attachments": [],
  "mentions": [{
    "id": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
    "name": "Sean Reed",
    "email": "sean.reed@postman.com",
    "references": ["1758013213559"],
    "notified": false,
    "entityId": null,
    "systemType": null
  }],
  "relatedRecords": null,
  "tasks": null,
  "associatedRecord": false,
  "gsAttachments": null,
  "new": true
}
```

**Response Schema:**
```json
{
  "data": {
    "id": "1I004SG7RDV06L1HFJ7H61L6DIHV8OPV25TE",
    "createdBy": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
    "createdDate": 1758013256980,
    "lastModifiedBy": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
    "lastModifiedDate": 1758013345719,
    "status": "EDITED",
    "version": 1,
    "note": {
      "type": "UPDATE",
      "subject": "Update",
      "content": "<p>@Sean Reed do we have a followup scheduled?</p>",
      "plainText": "@Sean Reed do we have a followup scheduled?",
      "posted": 1758013345781,
      "activityDate": 1758013200000
    },
    "contexts": [{
      "id": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
      "base": true,
      "obj": "Company",
      "lbl": "7 - Eleven (7 - 11) Inc."
    }],
    "mentions": [{
      "id": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
      "name": "Sean Reed",
      "email": "sean.reed@postman.com",
      "notified": false
    }]
  },
  "result": true,
  "code": null,
  "message": null,
  "requestId": "e408ae47-68e3-4307-933a-83bb211d3aa9"
}
```

### Activity Types Available:
- `UPDATE` - General note/update
- `MEETING` - Meeting notes with attendees
- `EMAIL` - Email activity log
- `CALL` - Phone call log
- `TASK` - Task creation with due dates

### Mention Format:
```html
<span data-profilepicture="null"
      data-value="Sean Reed"
      data-email="sean.reed@postman.com"
      data-name="Sean Reed"
      aria-label="Sean Reed"
      title="Sean Reed"
      data-reference="unique_reference_id"
      class="medium-editor-at-mention"
      data-id="USER_GSID">
  <span data-id="">@Sean Reed</span>
</span>
```

## Error Handling

### Common HTTP Status Codes:
- `200` - Success
- `401` - Unauthorized (token expired/invalid)
- `403` - Forbidden (insufficient permissions)
- `404` - Not found
- `429` - Rate limited
- `500` - Server error

### Error Response Schema:
```json
{
  "result": false,
  "code": "UNAUTHORIZED",
  "message": "Authentication token is invalid or expired",
  "requestId": "uuid",
  "additionalInfo": {
    "errorType": "AUTH_ERROR",
    "requiresReauth": true
  }
}
```

## Token/Session Management

### Token Refresh Mechanism
Based on HAR analysis, tokens are refreshed through:

```http
GET /v1/messenger/token
authtoken: {current_token}
Cookie: sid={session}
```

**Response:**
```json
{
  "result": true,
  "data": {
    "token": "new_jwt_token",
    "expiresIn": 3600,
    "refreshToken": "refresh_token_value"
  }
}
```

### Session Validation
```http
GET /v1/users/me
authtoken: {token}
Cookie: sid={session}
```

**Response for Valid Session:**
```json
{
  "result": true,
  "data": {
    "userId": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
    "name": "Jared Boynton",
    "email": "jared.boynton@postman.com",
    "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
    "permissions": ["READ", "WRITE", "ADMIN"]
  }
}
```

### WebSocket Connection (Optional)
For real-time updates:
```
WSS wss://prod-messenger.us2.gainsightapp.net/?id={jwt_token}
```

## Rate Limiting & Best Practices

### Rate Limits:
- **Search APIs**: 100 requests/minute
- **Timeline APIs**: 50 posts/hour
- **Report APIs**: 200 requests/minute

### Best Practices:
1. **Cache Company IDs** to avoid repeated searches
2. **Batch Timeline Queries** when possible
3. **Use WebSocket** for real-time timeline updates
4. **Implement exponential backoff** for 429 responses
5. **Validate tokens** before critical operations
6. **Log requestId** values for debugging

### Required Permissions:
- **Company Search**: `COMPANY_READ`
- **Report Access**: `REPORTS_READ`
- **Timeline Read**: `TIMELINE_READ`
- **Timeline Write**: `TIMELINE_WRITE`

## Complete End-to-End Workflow Example

### Scenario: Find "7-Eleven" company and post a timeline update

```bash
# Step 1: Search for company by partial name
curl -X POST "https://postman.us2.gainsightcloud.com/v1/data/objects/company/search?formatDate=true" \
  -H "Content-Type: application/json" \
  -H "authtoken: YOUR_JWT_TOKEN" \
  -H "Cookie: sid=YOUR_SESSION; platform_sid=YOUR_PLATFORM_SESSION" \
  -d '{
    "advanceFilters": null,
    "filters": [{"fields": ["Name"], "operator": "CONTAINS", "value": "7-11"}],
    "includeFields": ["Name"],
    "from": 0,
    "size": 20
  }'

# Response: {"result": true, "data": {"records": [{"Name": "7 - Eleven (7 - 11) Inc.", "Id": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR"}]}}

# Step 2: Get relationship ID for the company
curl -X POST "https://postman.us2.gainsightcloud.com/v1/data/objects/relationship/search?formatDate=true" \
  -H "Content-Type: application/json" \
  -H "authtoken: YOUR_JWT_TOKEN" \
  -H "Cookie: sid=YOUR_SESSION; platform_sid=YOUR_PLATFORM_SESSION" \
  -d '{
    "advanceFilters": null,
    "filters": [{"fields": ["CompanyId__gr.Name", "Name"], "operator": "CONTAINS", "value": "7-11"}],
    "includeFields": ["Name", "CompanyId__gr.Name", "Postman_Team_ID__gc", "Csm__gr.Name"],
    "from": 0,
    "size": 20
  }'

# Response: {"result": true, "data": {"records": [{"Id": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU", "Postman_Team_ID__gc": "68639"}]}}

# Step 3: View existing timeline entries
curl -X POST "https://postman.us2.gainsightcloud.com/v1/ant/timeline/search/gsactivities?page=0&size=5" \
  -H "Content-Type: application/json" \
  -H "authtoken: YOUR_JWT_TOKEN" \
  -H "Cookie: sid=YOUR_SESSION; platform_sid=YOUR_PLATFORM_SESSION" \
  -d '{
    "searchText": "",
    "quickSearch": {},
    "contextFilter": {"cids": ["1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR"]},
    "filterContext": null
  }'

# Step 4: Get report data (e.g., Product Usage)
curl -X POST "https://postman.us2.gainsightcloud.com/v3/bi/reporting/section/report-data?useCache=false&requestSource=R360&includeId=true&piedc=true&entityId=rId&c360RevampEnabled=true" \
  -H "Content-Type: application/json" \
  -H "authtoken: YOUR_JWT_TOKEN" \
  -H "Cookie: sid=YOUR_SESSION; platform_sid=YOUR_PLATFORM_SESSION" \
  -d '{
    "reportId": "83b5e7f7-87de-47f3-bb94-218ddf041dbb",
    "userId": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
    "widgetGlobalFilters": {
      "conditions": [{
        "leftOperand": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dataType": "LOOKUP",
          "objectName": "product_usage_team_metrics_weekly__gc"
        },
        "filterAlias": "bm_0",
        "logicalOperator": "AND",
        "comparisonOperator": "EQ",
        "rightOperandType": "VALUE",
        "filterValue": {"value": ["68639"]}
      }],
      "expression": "bm_0"
    }
  }'

# Step 5: Post to timeline with mention
curl -X POST "https://postman.us2.gainsightcloud.com/v1/ant/v2/activity" \
  -H "Content-Type: application/json" \
  -H "authtoken: YOUR_JWT_TOKEN" \
  -H "Cookie: sid=YOUR_SESSION; platform_sid=YOUR_PLATFORM_SESSION" \
  -d '{
    "note": {
      "type": "UPDATE",
      "subject": "Update",
      "activityDate": 1758013200000,
      "content": "<p>Customer check-in completed - API usage looks healthy</p>",
      "plainText": "Customer check-in completed - API usage looks healthy"
    },
    "contexts": [{
      "id": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
      "base": true,
      "obj": "Company",
      "lbl": "7 - Eleven (7 - 11) Inc."
    }],
    "author": {
      "id": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
      "name": "Your Name",
      "email": "your.email@postman.com"
    },
    "status": "POSTED",
    "new": true
  }'
```

### Key IDs and Their Purpose:

| ID Type | Example Value | Purpose |
|---------|---------------|---------|
| Company ID | `1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR` | Primary company identifier |
| Relationship ID | `1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU` | Links company to CSM/team |
| Team ID | `68639` | Postman team identifier for reporting |
| User ID | `1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA` | CSM user identifier |

### Implementation Flow in Code:

```javascript
async function fullGainsightWorkflow(companyName) {
  // 1. Search companies
  const companySearch = await fetch('/v1/data/objects/company/search?formatDate=true', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'authtoken': token },
    body: JSON.stringify({
      filters: [{ fields: ['Name'], operator: 'CONTAINS', value: companyName }],
      includeFields: ['Name'],
      from: 0, size: 20
    })
  });
  const company = await companySearch.json();
  const companyId = company.data.records[0].Id;

  // 2. Get relationship info
  const relationshipSearch = await fetch('/v1/data/objects/relationship/search?formatDate=true', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'authtoken': token },
    body: JSON.stringify({
      filters: [{ fields: ['CompanyId__gr.Name'], operator: 'EQ', value: companyName }],
      includeFields: ['Name', 'Postman_Team_ID__gc'],
      from: 0, size: 20
    })
  });
  const relationship = await relationshipSearch.json();
  const teamId = relationship.data.records[0]['Postman_Team_ID__gc'];

  // 3. Get timeline
  const timeline = await fetch('/v1/ant/timeline/search/gsactivities?page=0&size=5', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'authtoken': token },
    body: JSON.stringify({
      contextFilter: { cids: [companyId] },
      searchText: '', quickSearch: {}, filterContext: null
    })
  });

  // 4. Get reports (with team filter)
  const reportData = await fetch('/v3/bi/reporting/section/report-data', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'authtoken': token },
    body: JSON.stringify({
      reportId: 'your-report-id',
      widgetGlobalFilters: {
        conditions: [{
          leftOperand: { fieldName: 'Postman_Team_ID__gc' },
          comparisonOperator: 'EQ',
          filterValue: { value: [teamId] }
        }]
      }
    })
  });

  // 5. Post update
  const post = await fetch('/v1/ant/v2/activity', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json', 'authtoken': token },
    body: JSON.stringify({
      note: {
        type: 'UPDATE',
        subject: 'Automated Check-in',
        content: '<p>Weekly customer health check completed</p>',
        activityDate: Date.now()
      },
      contexts: [{ id: companyId, obj: 'Company', base: true }],
      status: 'POSTED',
      new: true
    })
  });

  return { company, relationship, timeline, reportData, post };
}
```

This guide provides the complete technical foundation for building Gainsight API integrations with proper error handling, authentication, and data schemas based on real HAR analysis.