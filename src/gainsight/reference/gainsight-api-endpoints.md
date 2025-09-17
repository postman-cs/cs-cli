# Gainsight Core API Endpoints

This document contains all API endpoints extracted from HAR analysis, excluding static assets.

## Authentication Requirements
All endpoints require the following headers:
```http
authtoken: [JWT token - required]
Cookie: sid=[session_id]; platform_sid=[platform_session]
Content-Type: application/json
x-gs-host: GAINSIGHT
```

---

## Authentication & User

### GET /v1/permissions/list-user-access
**URL:** `https://postman.us2.gainsightcloud.com/v1/permissions/list-user-access`
**Status Codes:** [200]
**Response Size:** 1385 - 1385 bytes

**Response:**
```json
{
  "result": true,
  "requestId": "cc9cf9e6-c94d-45a5-8802-3123064e862a",
  "data": {
    "userResourceDTOS": [
      {
        "name": "Jared Boynton",
        "id": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
        "sfdcId": "0051K00000A6WzsQAF",
        "email": "jared.boynton@postman.com",
        "superAdmin": false,
        "licenseType": "1I00DBNI8V79LWHWIV7RV125X155D476ON0S",
        "actionDetailsMap": {
          "apply_playbook": {
            "throughGroups": [
              {
                "entityId": "460af35f-f04b-36d2-8cf6-04b692ac2d80",
                "name": "CSM",
                "label": "CSM"
              }
            ],
            "throughResources": [
              null
            ],
            "throughReportees": [],
            "direct": false
          },
          "create_cta": {
            "throughGroups": [
              {
                "entityId": "460af35f-f04b-36d2-8cf6-04b692ac2d80",
                "name": "CSM",
                "label": "CSM"
              }
            ],
            "throughResources": [
              null
            ],
            "throughReportees": [],
            "direct": false
          },
          "delete_cta": {
            "throughGroups": [
              {
                "entityId": "460af35f-f04b-36d2-8cf6-04b692ac2d80",
                "name": "CSM",
                "label": "CSM"
              }
            ],
            "throughResources": [
              null
            ],
            "throughReportees": [],
            "direct": false
          }
        }
      }
    ],
    "publicResource": false
  }
}
```


### GET /v1/users/me
**URL:** `https://postman.us2.gainsightcloud.com/v1/users/me`
**Status Codes:** [200]
**Response Size:** 1709 - 1709 bytes

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "0ab96f20-28de-4d65-8b61-b1d45ecc4971",
  "data": {
    "Gsid": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
    "Name": "Jared Boynton",
    "FirstName": "Jared",
    "LastName": "Boynton",
    "Email": "jared.boynton@postman.com",
    "SFDCUserName": "jared.boynton@postman.com",
    "IsActiveUser": true,
    "IsSuperAdmin": false,
    "Manager": "1P01Q1IO46XBY7CLHN3BJGC8SLNXDC391OTM",
    "CompanyID": null,
    "SfdcUserId": "0051K00000A6WzsQAF",
    "GainsightLicenseEnabled": false,
    "Timezone": "America/Los_Angeles",
    "SystemType": "Internal",
    "Locale": "en_US",
    "Title": "Enterprise Customer Success Leader",
    "CreatedDate": "01/03/2025 23:27:54",
    "ModifiedDate": "09/15/2025 10:02:08",
    "CreatedBy": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
    "ModifiedBy": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
    "PreferredNotificationSource": "Gainsight NXT",
    "LicenseType": "Full",
    "IsPartner": false,
    "PartnerId": null,
    "OpenMini360": null,
    "Role": null,
    "sso_user_id": "1SUM543VGUGRLCQZHOPX1DHN54PX8WHF61KZ",
    "ProfilePicture": "images/3a5744bf-4121-4e2e-ab6f-ee7bf36fc524/1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA/v0/original.jpg",
    "identities": [
      {
        "crmId": null,
        "userId": "U02GTFD43QV",
        "type": "Slack",
        "appUrl": null,
        "connection": "Postman",
        "isPrimaryConnection": false,
        "connectionId": "T02G7V5JE"
      },
      {
        "crmId": null,
        "userId": "1520681781381",
        "type": "Zendesk",
        "appUrl": null,
        "connection": "Zendesk1",
        "isPrimaryConnection": false,
        "connectionId": "ebe92c2a-0c75-4546-b8af-21e9aea182de"
      },
      {
        "crmId": "SFDC_00D41000001Pq8IEAS",
        "userId": "0051K00000A6WzsQAF",
        "type": "Salesforce",
        "appUrl": "https://postman.my.salesforce.com/",
        "connection": "Salesforce",
        "isPrimaryConnection": true,
        "connectionId": "a238d19d-6851-44dd-9e28-38095d6d9ffa"
      }
    ]
  },
  "message": null,
  "localizedMessage": null
}
```


---

## Timeline & Activities

### GET /v1/ant//forms
**URL:** `https://postman.us2.gainsightcloud.com/v1/ant//forms?context=RelationshipType&contextId=1P06GH6NC0YFN2YXBDUFZX2FIJADKKBZ3AD3&showHidden=false`
**Query Parameters:** context, contextId, showHidden
**Status Codes:** [200]
**Response Size:** 29860 - 29860 bytes

**Response:**
```json
{
  "data": {
    "fieldLimit": 200,
    "fieldsMap": {
      "9c41ff26-c729-4b40-91b4-4395bac614b5": {
        "id": "9c41ff26-c729-4b40-91b4-4395bac614b5",
        "deleted": false,
        "name": "duration",
        "label": "Duration (in mins)",
        "description": "Time taken for the activity in minutes.",
        "dataType": "NUMBER",
        "fieldType": "NUMBER",
        "scope": "SYSTEM",
        "hidden": false,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "29b903c3-bd83-414f-b46f-194a18570855": {
        "id": "29b903c3-bd83-414f-b46f-194a18570855",
        "deleted": false,
        "name": "external_source",
        "label": "External Source",
        "description": "External Source",
        "dataType": "STRING",
        "fieldType": "TEXT",
        "scope": "SYSTEM",
        "hidden": true,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "e2e65a36-a2a1-4629-bc1f-011c5576c55e": {
        "id": "e2e65a36-a2a1-4629-bc1f-011c5576c55e",
        "deleted": false,
        "name": "milestoneType",
        "label": "Milestone Type",
        "description": "Milestone Type",
        "dataType": "ARRAY",
        "fieldType": "DROPDOWN",
        "scope": "SYSTEM",
        "hidden": true,
        "referTo": "SCRIBBLE",
        "picklistCategory": "MILESTONE",
        "decimalPlaces": 0,
        "da": true,
        "new": true
      },
      "58f7a853-672f-4ea3-992d-aeb46d1f73ad": {
        "id": "58f7a853-672f-4ea3-992d-aeb46d1f73ad",
        "deleted": false,
        "name": "Ant__Update_Type__c",
        "label": "Update Type",
        "dataType": "ARRAY",
        "fieldType": "DROPDOWN",
        "scope": "CUSTOM",
        "hidden": false,
        "picklistCategoryId": "1I00K0OXPFIU3ED4Y6GNN1W6LMCK3SOK5CD5",
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "98e1f30c-91ec-4e75-9f09-31060036a14c": {
        "id": "98e1f30c-91ec-4e75-9f09-31060036a14c",
        "deleted": false,
        "name": "Ant__Expected_Onboarding_Completion_Date__c",
        "label": "Expected Onboarding Completion Date",
        "dataType": "DATE",
        "fieldType": "DATE",
        "scope": "CUSTOM",
        "hidden": false,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "bf9ee034-0f31-4964-9e65-c324648d7c61": {
        "id": "bf9ee034-0f31-4964-9e65-c324648d7c61",
        "deleted": false,
        "name": "Ant__Level_of_Familiarity_with_Postman__c",
        "label": "Level of Familiarity with Postman",
        "dataType": "ARRAY",
        "fieldType": "DROPDOWN",
        "scope": "CUSTOM",
        "hidden": false,
        "picklistCategoryId": "1I00CQ776VT1I171IOP5EEC71VLZ2MH7M6LV",
        "picklistCategoryName": "Level of Complexity",
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "ffcf4735-bf14-4eb1-bb23-50aea1ef7d8e": {
        "id": "ffcf4735-bf14-4eb1-bb23-50aea1ef7d8e",
        "deleted": false,
        "name": "ant_type",
        "label": "Activity Type",
        "description": "User who logs the activity. This information is automatically captured.",
        "dataType": "STRING",
        "fieldType": "TEXT",
        "scope": "SYSTEM",
        "hidden": true,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "2465d5de-31aa-4025-b7d3-1687ad00cd4a": {
        "id": "2465d5de-31aa-4025-b7d3-1687ad00cd4a",
        "deleted": false,
        "name": "priority",
        "label": "Priority",
        "description": "Default Priority",
        "dataType": "ARRAY",
        "fieldType": "SELECT",
        "scope": "SYSTEM",
        "hidden": true,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "1f2e3b04-d7cf-42f2-a14d-6029ecb3de3c": {
        "id": "1f2e3b04-d7cf-42f2-a14d-6029ecb3de3c",
        "deleted": false,
        "name": "Ant__CSM_Sentiment__c",
        "label": "CSM Sentiment",
        "dataType": "ARRAY",
        "fieldType": "DROPDOWN",
        "scope": "CUSTOM",
        "hidden": false,
        "picklistCategoryId": "1I00AYEWKCFWHQELK82Q3PT8Q5ZQPW6EAA2C",
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "19c40be8-b3de-4e7f-abf0-c3847bdcc85c": {
        "id": "19c40be8-b3de-4e7f-abf0-c3847bdcc85c",
        "deleted": false,
        "name": "createdDate",
        "label": "Created Date",
        "description": "Date on which activity was logged.",
        "dataType": "DATETIME",
        "fieldType": "DATETIME",
        "scope": "SYSTEM",
        "hidden": true,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "0a01d10b-9163-48b3-ad2d-06caf5116278": {
        "id": "0a01d10b-9163-48b3-ad2d-06caf5116278",
        "deleted": false,
        "name": "internalAttendees",
        "label": "Internal Attendees",
        "description": "Internal users who were part of the activity.",
        "dataType": "STRING",
        "fieldType": "TAG",
        "scope": "SYSTEM",
        "hidden": false,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "9bc7082a-0bb6-4e13-a540-34d08d4f099c": {
        "id": "9bc7082a-0bb6-4e13-a540-34d08d4f099c",
        "deleted": false,
        "name": "linkTo",
        "label": "Link To",
        "description": "Account or Opportunity",
        "dataType": "STRING",
        "fieldType": "TEXT",
        "scope": "SYSTEM",
        "hidden": true,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "129445d0-a485-4564-9d67-e1a50b5d25e4": {
        "id": "129445d0-a485-4564-9d67-e1a50b5d25e4",
        "deleted": false,
        "name": "externalAttendees",
        "label": "External Attendees",
        "description": "External contacts who were part of the activity.",
        "dataType": "STRING",
        "fieldType": "TAG",
        "scope": "SYSTEM",
        "hidden": false,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "36783e08-c99c-48ae-af54-c8fd01025cf2": {
        "id": "36783e08-c99c-48ae-af54-c8fd01025cf2",
        "deleted": false,
        "name": "note",
        "label": "Note",
        "description": "Text area to display the description of the activity.",
        "dataType": "STRING",
        "fieldType": "RICH_TEXT_AREA",
        "scope": "SYSTEM",
        "hidden": false,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "f2c8cc04-7d56-4427-983a-2e099d544be8": {
        "id": "f2c8cc04-7d56-4427-983a-2e099d544be8",
        "deleted": false,
        "name": "Ant__Meeting_Category__c",
        "label": "Meeting Category",
        "dataType": "ARRAY",
        "fieldType": "DROPDOWN",
        "scope": "CUSTOM",
        "hidden": false,
        "picklistCategoryId": "1I00L7DBK9DJ91BCIOPAYNKR29JDP5N6GX7R",
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "8c538718-5598-4ecf-9841-0bceb40929b4": {
        "id": "8c538718-5598-4ecf-9841-0bceb40929b4",
        "deleted": false,
        "name": "subject",
        "label": "Subject",
        "description": "Text to capture the context or heading of the activity.",
        "dataType": "STRING",
        "fieldType": "TEXT",
        "scope": "SYSTEM",
        "hidden": false,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "557c1341-42d1-4e28-8096-9936251b64b6": {
        "id": "557c1341-42d1-4e28-8096-9936251b64b6",
        "deleted": false,
        "name": "associated_record",
        "label": "Associated Records",
        "description": "Associated Records Description",
        "dataType": "ARRAY",
        "fieldType": "DROPDOWN",
        "scope": "SYSTEM",
        "hidden": false,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "9e599a6f-6ea4-4b55-8742-7ba79e97f4fa": {
        "id": "9e599a6f-6ea4-4b55-8742-7ba79e97f4fa",
        "deleted": false,
        "name": "trackers",
        "label": "Trackers",
        "description": "Media Trackers",
        "dataType": "STRING",
        "fieldType": "TEXT",
        "scope": "SYSTEM",
        "hidden": true,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "1b561c60-702b-4e39-bd7e-516b32eefdfa": {
        "id": "1b561c60-702b-4e39-bd7e-516b32eefdfa",
        "deleted": false,
        "name": "activityDate",
        "label": "Activity Date",
        "description": "Activity Date",
        "dataType": "DATE",
        "fieldType": "DATE",
        "scope": "SYSTEM",
        "hidden": false,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "c3beb67b-f63f-4be4-8371-f8989095e755": {
        "id": "c3beb67b-f63f-4be4-8371-f8989095e755",
        "deleted": false,
        "name": "media_url",
        "label": "Media URL",
        "description": "Media URL of audio/video",
        "dataType": "STRING",
        "fieldType": "TEXT",
        "scope": "SYSTEM",
        "hidden": true,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "1d99fbe9-171a-4c61-a468-4b0ed9a7a1ba": {
        "id": "1d99fbe9-171a-4c61-a468-4b0ed9a7a1ba",
        "deleted": false,
        "name": "createdBy",
        "label": "Created By",
        "description": "Created By",
        "dataType": "STRING",
        "fieldType": "TEXT",
        "scope": "SYSTEM",
        "hidden": true,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "7130434f-2f08-4286-861d-edfe93a051aa": {
        "id": "7130434f-2f08-4286-861d-edfe93a051aa",
        "deleted": false,
        "name": "Ant__Use_Case_Alignment_to_Customer_Goals__c",
        "label": "Use Case Alignment to Customer Goals",
        "dataType": "ARRAY",
        "fieldType": "DROPDOWN",
        "scope": "CUSTOM",
        "hidden": false,
        "picklistCategoryId": "1I00NE5SLFOWW67BQB6184C5AJDRL06RCW89",
        "picklistCategoryName": "Alignment",
        "decimalPlaces": 0,
        "da": false,
        "new": true
      },
      "97b2957d-6f83-4879-8f1e-2bafb96367d0": {
        "id": "97b2957d-6f83-4879-8f1e-2bafb96367d0",
        "deleted": false,
        "name": "status",
        "label": "Default Closed Status",
        "description": "Default Closed Status",
        "dataType": "ARRAY",
        "fieldType": "SELECT",
        "scope": "SYSTEM",
        "hidden": true,
        "decimalPlaces": 0,
        "da": false,
        "new": true
      }
    },
    "reportingCategories": [
      {
        "rcId": "921e7705-ff99-4f4d-89b1-518e0f653205",
        "name": "Update",
        "label": "Update"
      },
      {
        "rcId": "c9589269-c8a6-4e9b-827b-f4bf5bb82dd1",
        "name": "Call",
        "label": "Call"
      },
      {
        "rcId": "416cf72a-a0a5-492c-b9d6-c6e4fafbccff",
        "name": "Meeting",
        "label": "Meeting"
      },
      "... [truncated]"
    ],
    "formFieldLimit": 15,
    "formLimit": 50,
    "antIntegrationEnabled": true,
    "hideReparentingConfirmationDialog": false,
    "activityTypes": [
      {
        "id": "056290d2-4e11-46df-9795-2b223cca0c20",
        "createdBy": null,
        "createdDate": 1755539530822,
        "lastModifiedBy": null,
        "lastModifiedDate": 1755539530822,
        "sysModTimestamp": null,
        "deleted": false,
        "version": 8,
        "createdByUser": {
          "gsId": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
          "name": null,
          "pp": null,
          "eid": "005Rf0000036HUHIA2",
          "esys": "SALESFORCE"
        },
        "lastModifiedByUser": {
          "gsId": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
          "name": null,
          "pp": null,
          "eid": "005Rf0000036HUHIA2",
          "esys": "SALESFORCE"
        },
        "formId": "056290d2-4e11-46df-9795-2b223cca0c20",
        "name": "Update",
        "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
        "type": "UPDATE",
        "syncToSalesforce": false,
        "icon": "message",
        "fieldDefinitions": [
          {
            "id": "8c538718-5598-4ecf-9841-0bceb40929b4",
            "displayOrder": 1,
            "required": true,
            "displayName": "Subject",
            "mandatory": true,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "36783e08-c99c-48ae-af54-c8fd01025cf2",
            "displayOrder": 2,
            "required": true,
            "displayName": "Note",
            "mandatory": true,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "1b561c60-702b-4e39-bd7e-516b32eefdfa",
            "displayOrder": 3,
            "required": true,
            "displayName": "Activity Date",
            "mandatory": true,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "f2c8cc04-7d56-4427-983a-2e099d544be8",
            "displayOrder": 4,
            "required": false,
            "displayName": "Update Category",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "129445d0-a485-4564-9d67-e1a50b5d25e4",
            "displayOrder": 5,
            "required": false,
            "displayName": "External Attendees",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "557c1341-42d1-4e28-8096-9936251b64b6",
            "displayOrder": 6,
            "required": false,
            "displayName": "Associated Records",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          }
        ],
        "activityText": "has logged a Update",
        "context": null,
        "contextId": null,
        "displayOrder": 1,
        "referenceFormId": null,
        "active": true,
        "activityUsageCount": 0,
        "draftUsageCount": 0,
        "scope": "SYSTEM",
        "hidden": false,
        "showInFilter": true,
        "rcId": "921e7705-ff99-4f4d-89b1-518e0f653205",
        "includeContexts": [
          "COMPANY"
        ],
        "excludeContexts": [],
        "autoAssign": true,
        "defaultType": false,
        "color": "#fff2f1",
        "new": false,
        "extContext": null,
        "extContextId": null,
        "extSystem": null
      },
      {
        "id": "131b6b5a-2441-4580-8acd-7142852ef592",
        "createdBy": null,
        "createdDate": 1730457672642,
        "lastModifiedBy": null,
        "lastModifiedDate": 1738082191966,
        "sysModTimestamp": null,
        "deleted": false,
        "version": 1,
        "createdByUser": {
          "gsId": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
          "name": null,
          "pp": null,
          "eid": "",
          "esys": "SALESFORCE"
        },
        "lastModifiedByUser": {
          "gsId": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
          "name": null,
          "pp": null,
          "eid": "005Rf0000036HUHIA2",
          "esys": "SALESFORCE"
        },
        "formId": "131b6b5a-2441-4580-8acd-7142852ef592",
        "name": "Call",
        "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
        "type": "CALL",
        "syncToSalesforce": false,
        "icon": "activity-call",
        "fieldDefinitions": [
          {
            "id": "8c538718-5598-4ecf-9841-0bceb40929b4",
            "displayOrder": 1,
            "required": true,
            "displayName": "Subject",
            "mandatory": true,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "1b561c60-702b-4e39-bd7e-516b32eefdfa",
            "displayOrder": 2,
            "required": true,
            "displayName": "Activity Date",
            "mandatory": true,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "9c41ff26-c729-4b40-91b4-4395bac614b5",
            "displayOrder": 3,
            "required": true,
            "displayName": "Duration (in mins)",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "0a01d10b-9163-48b3-ad2d-06caf5116278",
            "displayOrder": 4,
            "required": true,
            "displayName": "Internal Attendees",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "129445d0-a485-4564-9d67-e1a50b5d25e4",
            "displayOrder": 5,
            "required": true,
            "displayName": "External Attendees",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "36783e08-c99c-48ae-af54-c8fd01025cf2",
            "displayOrder": 6,
            "required": true,
            "displayName": "Note",
            "mandatory": true,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "557c1341-42d1-4e28-8096-9936251b64b6",
            "displayOrder": 7,
            "required": false,
            "displayName": "Associated Records",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          }
        ],
        "activityText": "Had a call.",
        "context": "Global",
        "contextId": null,
        "displayOrder": 2,
        "referenceFormId": null,
        "active": false,
        "activityUsageCount": 0,
        "draftUsageCount": 0,
        "scope": "SYSTEM",
        "hidden": false,
        "showInFilter": true,
        "rcId": "c9589269-c8a6-4e9b-827b-f4bf5bb82dd1",
        "includeContexts": [
          "COMPANY"
        ],
        "excludeContexts": [],
        "autoAssign": true,
        "defaultType": false,
        "color": "#d3ecfa",
        "new": false,
        "extContext": "Global",
        "extContextId": null,
        "extSystem": null
      },
      {
        "id": "93ea9440-ad74-4011-957d-3236c1d084ce",
        "createdBy": null,
        "createdDate": 1757339685740,
        "lastModifiedBy": null,
        "lastModifiedDate": 1757339685740,
        "sysModTimestamp": null,
        "deleted": false,
        "version": 7,
        "createdByUser": {
          "gsId": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
          "name": null,
          "pp": null,
          "eid": "005Rf0000036HUHIA2",
          "esys": "SALESFORCE"
        },
        "lastModifiedByUser": {
          "gsId": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
          "name": null,
          "pp": null,
          "eid": "005Rf0000036HUHIA2",
          "esys": "SALESFORCE"
        },
        "formId": "93ea9440-ad74-4011-957d-3236c1d084ce",
        "name": "Meeting",
        "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
        "type": "MEETING",
        "syncToSalesforce": false,
        "icon": "calendar-marked",
        "fieldDefinitions": [
          {
            "id": "36783e08-c99c-48ae-af54-c8fd01025cf2",
            "displayOrder": 1,
            "required": true,
            "displayName": "Note",
            "mandatory": true,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "8c538718-5598-4ecf-9841-0bceb40929b4",
            "displayOrder": 2,
            "required": true,
            "displayName": "Subject",
            "mandatory": true,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "1b561c60-702b-4e39-bd7e-516b32eefdfa",
            "displayOrder": 3,
            "required": true,
            "displayName": "Activity Date",
            "mandatory": true,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "9c41ff26-c729-4b40-91b4-4395bac614b5",
            "displayOrder": 4,
            "required": true,
            "displayName": "Duration (in mins)",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "0a01d10b-9163-48b3-ad2d-06caf5116278",
            "displayOrder": 5,
            "required": true,
            "displayName": "Internal Attendees",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "129445d0-a485-4564-9d67-e1a50b5d25e4",
            "displayOrder": 6,
            "required": true,
            "displayName": "External Attendees",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "f2c8cc04-7d56-4427-983a-2e099d544be8",
            "displayOrder": 7,
            "required": false,
            "displayName": "Meeting Category",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          },
          {
            "id": "557c1341-42d1-4e28-8096-9936251b64b6",
            "displayOrder": 8,
            "required": false,
            "displayName": "Associated Records",
            "mandatory": false,
            "hidden": false,
            "disabled": false,
            "controllerFieldId": null,
            "decimalPlaces": 0,
            "defaultValue": null,
            "fieldAccess": "WRITE"
          }
        ],
        "activityText": "has logged a Meeting",
        "context": null,
        "contextId": null,
        "displayOrder": 3,
        "referenceFormId": null,
        "active": true,
        "activityUsageCount": 0,
        "draftUsageCount": 0,
        "scope": "SYSTEM",
        "hidden": false,
        "showInFilter": true,
        "rcId": "416cf72a-a0a5-492c-b9d6-c6e4fafbccff",
        "includeContexts": [
          "COMPANY"
        ],
        "excludeContexts": [],
        "autoAssign": true,
        "defaultType": false,
        "color": "#d4f0ed",
        "new": false,
        "extContext": null,
        "extContextId": null,
        "extSystem": null
      },
      "... [truncated]"
    ],
    "isScribbleEnabled": true
  },
  "result": true,
  "code": null,
  "message": null,
  "additionalInfo": null,
  "requestId": "de954d7e-bb45-4d89-9b6f-7a4b4ba72aae"
}
```


### GET /v1/ant/v2/activity/user/permissions
**URL:** `https://postman.us2.gainsightcloud.com/v1/ant/v2/activity/user/permissions`
**Status Codes:** [200]
**Response Size:** 250 - 250 bytes

**Response:**
```json
{
  "data": {
    "userPermissions": {
      "canEdit": false,
      "canDelete": false
    },
    "defaultSystemAdministratorGsid": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F"
  },
  "result": true,
  "code": null,
  "message": null,
  "additionalInfo": null,
  "requestId": "85e83ec7-a06d-4580-80ac-46787def955b"
}
```


### POST /v1/ant/timeline/search/gsactivities
**URL:** `https://postman.us2.gainsightcloud.com/v1/ant/timeline/search/gsactivities?page=0&size=5`
**Query Parameters:** page, size
**Status Codes:** [200]
**Response Size:** 18148 - 18148 bytes

**Request Body:**
```json
{
  "searchText": "",
  "quickSearch": {},
  "contextFilter": {
    "rids": [
      "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU"
    ]
  },
  "filterContext": null
}
```

**Response:**
```json
{
  "data": {
    "links": [
      {
        "rel": "first",
        "href": "https://postman.us2.gainsightcloud.com/v1/ant/timeline/search/gsactivities?page=0&size=5"
      },
      {
        "rel": "last",
        "href": "https://postman.us2.gainsightcloud.com/v1/ant/timeline/search/gsactivities?page=0&size=5"
      },
      {
        "rel": "self",
        "href": "https://postman.us2.gainsightcloud.com/v1/ant/timeline/search/gsactivities?page=0&size=5"
      }
    ],
    "page": {
      "number": 0,
      "size": 3,
      "totalPages": 2,
      "totalElements": 10
    },
    "content": [
      {
        "id": "1I004SG7RDV06L1HFJPU21ZG48X58N2Y6J07",
        "createdBy": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
        "createdDate": 1753403247930,
        "lastModifiedBy": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
        "lastModifiedDate": 1753403247930,
        "sysModTimestamp": null,
        "deleted": false,
        "version": 0,
        "createdByUser": null,
        "lastModifiedByUser": {
          "gsId": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
          "name": "Sean Reed",
          "pp": "",
          "eid": "",
          "esys": ""
        },
        "note": {
          "type": "UPDATE",
          "subject": "Weekly Update 7/25",
          "content": "&lt;p&gt;&lt;b&gt;What has happened since the last update?&lt;/b&gt;&lt;/p&gt;&lt;p&gt;&lt;/p&gt;&lt;ul&gt;&lt;li&gt;Fix for requested anonymization of (de-)provisioning emails has been pushed. Customer notified (https://postman.zendesk.com/agent/tickets/250176)&lt;/li&gt;&lt;/ul&gt;&lt;p&gt;&lt;b&gt;What is happening next?&lt;/b&gt;&lt;/p&gt;&lt;p&gt;&lt;/p&gt;&lt;ul&gt;&lt;li&gt;Meeting between Karthik &amp;amp; Jared/Eric. We&#x27;ll be aiming to understand their API strategy for development, discovery, and promotion to production (Tuesday 7/29)&lt;/li&gt;&lt;/ul&gt;&lt;p&gt;&lt;b&gt;&lt;br&gt;&lt;/b&gt;&lt;/p&gt;&lt;p&gt;&lt;b color=&quot;&quot; style=&quot;font-family: Roboto, -apple-system, BlinkMacSystemFont,&quot;&gt;Who is it happening with?&lt;/b&gt;&lt;/p&gt;&lt;p&gt;&lt;/p&gt;&lt;p&gt;&lt;span color=&quot;&quot; style=&quot;font-family: Roboto, -apple-system, BlinkMacSystemFont,&quot;&gt;- Karthik, Eric, Jared&lt;/span&gt;&lt;/p&gt;&lt;p&gt;&lt;b color=&quot;&quot; style=&quot;font-family: Roboto, -apple-system, BlinkMacSystemFont,&quot;&gt;&lt;br&gt;&lt;/b&gt;&lt;/p&gt;&lt;p&gt;&lt;b&gt;Are there any blockers to progress? If so, what help is needed?&lt;/b&gt;&lt;/p&gt;",
          "plainText": "What has happened since the last update? Fix for requested anonymization of (de-)provisioning emails has been pushed. Customer notified (https://postman.zendesk.com/agent/tickets/250176) What is happening next? Meeting between Karthik & Jared/Eric. We'll be aiming to understand their API strategy for development, discovery, and promotion to production (Tuesday 7/29) Who is it happening with? - Karthik, Eric, Jared Are there any blockers to progress? If so, what help is needed?",
          "posted": 1758014391569,
          "activityDate": 1753402800000,
          "status": null,
          "customFields": {
            "duration": null,
            "Ant__Meeting_Category__c": null,
            "internalAttendees": [],
            "externalAttendees": []
          },
          "duration": null,
          "internalAttendees": null,
          "externalAttendees": null,
          "trackers": [],
          "important": false
        },
        "context": null,
        "contexts": [
          {
            "id": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
            "base": false,
            "obj": "Company",
            "lbl": "7 - Eleven (7 - 11) Inc.",
            "eid": "0011K00002H9qNmQAJ",
            "eobj": "Account",
            "eurl": null,
            "esys": "SALESFORCE",
            "dsp": true
          },
          {
            "id": "1P06GH6NC0YFN2YXBDUFZX2FIJADKKBZ3AD3",
            "base": false,
            "obj": "RelationshipType",
            "lbl": "Postman Team",
            "eid": null,
            "eobj": "RelationshipType",
            "eurl": null,
            "esys": "SALESFORCE",
            "dsp": false
          },
          {
            "id": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU",
            "base": true,
            "obj": "Relationship",
            "lbl": "7-Eleven - Ent 68639 - 10/1 - 68639",
            "eid": "a3o1K000001o9rKQAQ",
            "eobj": "Relationship",
            "eurl": null,
            "esys": "SALESFORCE",
            "dsp": true
          }
        ],
        "author": {
          "userId": null,
          "userName": null,
          "userEmail": null,
          "profile": null,
          "userType": null,
          "systemType": null,
          "id": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
          "name": "Sean Reed",
          "email": "sean.reed@postman.com",
          "companyId": null,
          "licenseType": null,
          "createdDate": null,
          "activeUser": false,
          "obj": null,
          "eid": "005Rf000002zZanIAE",
          "eobj": null,
          "eurl": null,
          "esys": null,
          "epp": null,
          "pp": null
        },
        "status": "POSTED",
        "tags": null,
        "attachments": [],
        "meta": {
          "color": null,
          "comments": 0,
          "likes": 0,
          "chars": 0,
          "size": 0,
          "edits": 0,
          "attachments": 0,
          "activityTypeId": "056290d2-4e11-46df-9795-2b223cca0c20",
          "ctaId": null,
          "scoreMetaId": null,
          "systemGenerated": false,
          "source": "R360",
          "externalSource": null,
          "mediaUrl": null,
          "reportSynced": false,
          "sfdcEventId": null,
          "countOfSyncFailures": 0,
          "hasTask": false,
          "emailSent": false,
          "notesTemplateId": "1I0083FH2QFON7PSJHI48H4DLB6APADH51DC",
          "activitySentiment": null,
          "genaiSentimentRationale": null,
          "genaiSentimentVerdictReasons": null,
          "genaiStatus": "NA",
          "externalSourceDetails": null,
          "comic": 0,
          "eid": null
        },
        "sfdcTaskId": null,
        "syncedToSFDC": false,
        "formMetadata": null,
        "mentions": [],
        "relatedRecords": null,
        "tasks": null,
        "associatedRecord": false,
        "gsAttachments": null,
        "new": false
      },
      {
        "id": "1I004SG7RDV06L1HFJKNTZF6U8IKLVR9DCFG",
        "createdBy": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
        "createdDate": 1751412304234,
        "lastModifiedBy": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
        "lastModifiedDate": 1751412304234,
        "sysModTimestamp": null,
        "deleted": false,
        "version": 0,
        "createdByUser": null,
        "lastModifiedByUser": {
          "gsId": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
          "name": "Sean Reed",
          "pp": "",
          "eid": "",
          "esys": ""
        },
        "note": {
          "type": "UPDATE",
          "subject": "Weekly Update - 7/4",
          "content": "&lt;p&gt;Objective: Set a meeting with Karthik to understand current state of 7-11 as a postman customer. Up to this point, we&#x27;ve been non-strategic with this account\u2014 solving for access issues and admin updates.&amp;nbsp;&lt;/p&gt;&lt;p&gt;&lt;br&gt;&lt;/p&gt;&lt;p&gt;They have been a slow-growth steady customer and expansion should continue, but risk profile is not yet understood.&amp;nbsp;&lt;/p&gt;&lt;p&gt;&lt;br&gt;&lt;/p&gt;&lt;p&gt;Sean to establish sync with Karthik. Need to get a state of the customer and ensure that we&#x27;re enabling strategic objectives.&amp;nbsp;&lt;/p&gt;",
          "plainText": "Objective: Set a meeting with Karthik to understand current state of 7-11 as a postman customer. Up to this point, we've been non-strategic with this account\u2014 solving for access issues and admin updates. They have been a slow-growth steady customer and expansion should continue, but risk profile is not yet understood. Sean to establish sync with Karthik. Need to get a state of the customer and ensure that we're enabling strategic objectives.",
          "posted": 1758014391569,
          "activityDate": 1751412060000,
          "status": null,
          "customFields": {
            "duration": null,
            "Ant__Meeting_Category__c": null,
            "internalAttendees": [],
            "externalAttendees": [
              {
                "userId": null,
                "userName": null,
                "userEmail": null,
                "profile": null,
                "userType": "CONTACT",
                "systemType": null,
                "id": "1C01QVIMIKMMFNKZ27WVOYJTOMPA6X053WQ2",
                "name": "Karthik Turaga",
                "email": "karthik.turaga@7-11.com",
                "companyId": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
                "licenseType": null,
                "createdDate": null,
                "attendeeType": null,
                "activityId": "1I004SG7RDV06L1HFJKNTZF6U8IKLVR9DCFG",
                "personId": null,
                "relationshipPersonId": null,
                "gs_internalattendeeid": null,
                "gs_externalattendeeid": null,
                "externalResolutionId": null,
                "activityExternalId": null,
                "firstName": null,
                "lastName": null,
                "deleted": false,
                "activeUser": false,
                "obj": null,
                "eid": null,
                "eobj": null,
                "eurl": null,
                "esys": null,
                "epp": null,
                "pp": null
              }
            ]
          },
          "duration": null,
          "internalAttendees": null,
          "externalAttendees": null,
          "trackers": [],
          "important": false
        },
        "context": null,
        "contexts": [
          {
            "id": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
            "base": false,
            "obj": "Company",
            "lbl": "7 - Eleven (7 - 11) Inc.",
            "eid": "0011K00002H9qNmQAJ",
            "eobj": "Account",
            "eurl": null,
            "esys": "SALESFORCE",
            "dsp": true
          },
          {
            "id": "1P06GH6NC0YFN2YXBDUFZX2FIJADKKBZ3AD3",
            "base": false,
            "obj": "RelationshipType",
            "lbl": "Postman Team",
            "eid": null,
            "eobj": "RelationshipType",
            "eurl": null,
            "esys": "SALESFORCE",
            "dsp": false
          },
          {
            "id": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU",
            "base": true,
            "obj": "Relationship",
            "lbl": "7-Eleven - Ent 68639 - 10/1 - 68639",
            "eid": "a3o1K000001o9rKQAQ",
            "eobj": "Relationship",
            "eurl": null,
            "esys": "SALESFORCE",
            "dsp": true
          }
        ],
        "author": {
          "userId": null,
          "userName": null,
          "userEmail": null,
          "profile": null,
          "userType": null,
          "systemType": null,
          "id": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
          "name": "Sean Reed",
          "email": "sean.reed@postman.com",
          "companyId": null,
          "licenseType": null,
          "createdDate": null,
          "activeUser": false,
          "obj": null,
          "eid": "005Rf000002zZanIAE",
          "eobj": null,
          "eurl": null,
          "esys": null,
          "epp": null,
          "pp": null
        },
        "status": "POSTED",
        "tags": null,
        "attachments": [],
        "meta": {
          "color": null,
          "comments": 0,
          "likes": 0,
          "chars": 0,
          "size": 0,
          "edits": 0,
          "attachments": 0,
          "activityTypeId": "056290d2-4e11-46df-9795-2b223cca0c20",
          "ctaId": "1S017HUYN5ABG8Z6CC38OHV1CYNWIBYRPGZY",
          "scoreMetaId": null,
          "systemGenerated": false,
          "source": "R360",
          "externalSource": null,
          "mediaUrl": null,
          "reportSynced": false,
          "sfdcEventId": null,
          "countOfSyncFailures": 0,
          "hasTask": true,
          "emailSent": false,
          "notesTemplateId": null,
          "activitySentiment": null,
          "genaiSentimentRationale": null,
          "genaiSentimentVerdictReasons": null,
          "genaiStatus": "NA",
          "externalSourceDetails": null,
          "comic": 0,
          "eid": null
        },
        "sfdcTaskId": null,
        "syncedToSFDC": false,
        "formMetadata": null,
        "mentions": [],
        "relatedRecords": null,
        "tasks": null,
        "associatedRecord": false,
        "gsAttachments": null,
        "new": false
      },
      {
        "id": "1I004SG7RDV06L1HFJ523WSZF1K9YBCMG5HF",
        "createdBy": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
        "createdDate": 1744063311968,
        "lastModifiedBy": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
        "lastModifiedDate": 1751413520889,
        "sysModTimestamp": null,
        "deleted": false,
        "version": 1,
        "createdByUser": null,
        "lastModifiedByUser": {
          "gsId": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
          "name": "Sean Reed",
          "pp": "",
          "eid": "",
          "esys": ""
        },
        "note": {
          "type": "MEETING",
          "subject": " Postman SSO Discussion",
          "content": "&lt;p&gt;&lt;/p&gt;&lt;p style=&quot;line-height:1.38;margin-top:0pt;margin-bottom:0pt;&quot;&gt;Attendees \u2013 Chiranjeevi, Zeeshan Ali, Karthik Turaga&lt;/p&gt;&lt;ul style=&quot;margin-bottom: 0px; padding-inline-start: 48px;&quot;&gt;&lt;li aria-level=&quot;1&quot; style=&quot;font-size: 11pt; font-family: Arial, sans-serif; color: rgb(0, 0, 0); font-variant-numeric: normal; font-variant-east-asian: normal; font-variant-alternates: normal; font-variant-position: normal; font-variant-emoji: normal; vertical-align: baseline; white-space: pre;&quot;&gt;&lt;p role=&quot;presentation&quot; style=&quot;line-height:1.38;margin-top:0pt;margin-bottom:0pt;&quot;&gt;Move from Ping to Azure ID&lt;/p&gt;&lt;/li&gt;&lt;li aria-level=&quot;1&quot; style=&quot;font-size: 11pt; font-family: Arial, sans-serif; color: rgb(0, 0, 0); font-variant-numeric: normal; font-variant-east-asian: normal; font-variant-alternates: normal; font-variant-position: normal; font-variant-emoji: normal; vertical-align: baseline; white-space: pre;&quot;&gt;&lt;p role=&quot;presentation&quot; style=&quot;line-height:1.38;margin-top:0pt;margin-bottom:0pt;&quot;&gt;Currently Ping is still in place\u2026 Everyone who was in Ping is now in Azure&lt;/p&gt;&lt;/li&gt;&lt;li aria-level=&quot;1&quot; style=&quot;font-size: 11pt; font-family: Arial, sans-serif; color: rgb(0, 0, 0); font-variant-numeric: normal; font-variant-east-asian: normal; font-variant-alternates: normal; font-variant-position: normal; font-variant-emoji: normal; vertical-align: baseline; white-space: pre;&quot;&gt;&lt;p role=&quot;presentation&quot; style=&quot;line-height:1.38;margin-top:0pt;margin-bottom:0pt;&quot;&gt;Unfortunately, in Postman, user accounts were being duplicated due to differences in Emails listed in IDP&lt;/p&gt;&lt;/li&gt;&lt;li aria-level=&quot;1&quot; style=&quot;font-size: 11pt; font-family: Arial, sans-serif; color: rgb(0, 0, 0); font-variant-numeric: normal; font-variant-east-asian: normal; font-variant-alternates: normal; font-variant-position: normal; font-variant-emoji: normal; vertical-align: baseline; white-space: pre;&quot;&gt;&lt;p role=&quot;presentation&quot; style=&quot;line-height:1.38;margin-top:0pt;margin-bottom:0pt;&quot;&gt;Troubleshoot &lt;/p&gt;&lt;/li&gt;&lt;ul style=&quot;margin-bottom: 0px; padding-inline-start: 48px;&quot;&gt;&lt;li aria-level=&quot;2&quot; style=&quot;font-size: 11pt; font-family: Arial, sans-serif; color: rgb(0, 0, 0); font-variant-numeric: normal; font-variant-east-asian: normal; font-variant-alternates: normal; font-variant-position: normal; font-variant-emoji: normal; vertical-align: baseline; white-space: pre;&quot;&gt;&lt;p role=&quot;presentation&quot; style=&quot;line-height:1.38;margin-top:0pt;margin-bottom:0pt;&quot;&gt;Disable JIT&lt;/p&gt;&lt;/li&gt;&lt;li aria-level=&quot;2&quot; style=&quot;font-size: 11pt; font-family: Arial, sans-serif; color: rgb(0, 0, 0); font-variant-numeric: normal; font-variant-east-asian: normal; font-variant-alternates: normal; font-variant-position: normal; font-variant-emoji: normal; vertical-align: baseline; white-space: pre;&quot;&gt;&lt;p role=&quot;presentation&quot; style=&quot;line-height:1.38;margin-top:0pt;margin-bottom:0pt;&quot;&gt;Make all emails lower case in Postman&lt;/p&gt;&lt;/li&gt;&lt;li aria-level=&quot;2&quot; style=&quot;font-size: 11pt; font-family: Arial, sans-serif; color: rgb(0, 0, 0); font-variant-numeric: normal; font-variant-east-asian: normal; font-variant-alternates: normal; font-variant-position: normal; font-variant-emoji: normal; vertical-align: baseline; white-space: pre;&quot;&gt;&lt;p role=&quot;presentation&quot; style=&quot;line-height:1.38;margin-top:0pt;margin-bottom:0pt;&quot;&gt;THEN enable Azure w/ lowercase transformation&lt;/p&gt;&lt;/li&gt;&lt;/ul&gt;&lt;/ul&gt;&lt;p&gt;&lt;font face=&quot;Arial, sans-serif&quot; color=&quot;#000000&quot;&gt;&lt;span style=&quot;font-size: 14.6667px; white-space: pre;&quot;&gt;&lt;br&gt;&lt;/span&gt;&lt;/font&gt;&lt;/p&gt;&lt;p&gt;&lt;font face=&quot;Arial, sans-serif&quot; color=&quot;#000000&quot;&gt;&lt;span style=&quot;font-size: 14.6667px; white-space: pre;&quot;&gt;Primary issue: capital-/lower-case email variations are creating issues with accounts during the transition to Azure SSO&lt;/span&gt;&lt;/font&gt;&lt;/p&gt;&lt;p&gt;&lt;font face=&quot;Arial, sans-serif&quot; color=&quot;#000000&quot;&gt;&lt;span style=&quot;font-size: 14.6667px; white-space: pre;&quot;&gt;Support thread started here: &lt;/span&gt;&lt;/font&gt;&lt;span color=&quot;&quot; style=&quot;font-family: Roboto, -apple-system, BlinkMacSystemFont,&quot;&gt;&lt;a target=&quot;_blank&quot; href=&quot;https://postman.zendesk.com/agent/tickets/239899&quot;&gt;&lt;span data-href=&quot;https://postman.zendesk.com/agent/tickets/239899&quot; data-auto-link=&quot;true&quot;&gt;https://postman.zendesk.com/agent/tickets/239899&lt;/span&gt;&lt;/a&gt;&lt;/span&gt;&lt;/p&gt;",
          "plainText": "Attendees \u2013 Chiranjeevi, Zeeshan Ali, Karthik Turaga Move from Ping to Azure ID Currently Ping is still in place\u2026 Everyone who was in Ping is now in Azure Unfortunately, in Postman, user accounts were being duplicated due to differences in Emails listed in IDP Troubleshoot Disable JIT Make all emails lower case in Postman THEN enable Azure w/ lowercase transformation Primary issue: capital-/lower-case email variations are creating issues with accounts during the transition to Azure SSO Support thread started here: https://postman.zendesk.com/agent/tickets/239899",
          "posted": 1758014391569,
          "activityDate": 1743631140000,
          "status": null,
          "customFields": {
            "duration": null,
            "Ant__Meeting_Category__c": "1I00XQI7RPUGJYHW5MD2QFJ5IUG0RQ8MF4SK",
            "internalAttendees": [
              {
                "userId": null,
                "userName": null,
                "userEmail": null,
                "profile": null,
                "userType": "USER",
                "systemType": null,
                "id": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
                "name": "Sean Reed",
                "email": "sean.reed@postman.com",
                "companyId": null,
                "licenseType": null,
                "createdDate": null,
                "activeUser": false,
                "obj": null,
                "eid": "005Rf000002zZanIAE",
                "eobj": null,
                "eurl": null,
                "esys": null,
                "epp": null,
                "pp": null
              }
            ],
            "externalAttendees": [
              {
                "userId": null,
                "userName": null,
                "userEmail": null,
                "profile": null,
                "userType": "CONTACT",
                "systemType": null,
                "id": "1C01QVIMIKMMFNKZ27WVOYJTOMPA6X053WQ2",
                "name": "Karthik Turaga",
                "email": "karthik.turaga@7-11.com",
                "companyId": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
                "licenseType": null,
                "createdDate": null,
                "attendeeType": null,
                "activityId": "1I004SG7RDV06L1HFJ523WSZF1K9YBCMG5HF",
                "personId": null,
                "relationshipPersonId": null,
                "gs_internalattendeeid": null,
                "gs_externalattendeeid": null,
                "externalResolutionId": null,
                "activityExternalId": null,
                "firstName": null,
                "lastName": null,
                "deleted": false,
                "activeUser": false,
                "obj": null,
                "eid": null,
                "eobj": null,
                "eurl": null,
                "esys": null,
                "epp": null,
                "pp": null
              }
            ]
          },
          "duration": null,
          "internalAttendees": null,
          "externalAttendees": null,
          "trackers": [],
          "important": false
        },
        "context": null,
        "contexts": [
          {
            "id": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
            "base": false,
            "obj": "Company",
            "lbl": "7 - Eleven (7 - 11) Inc.",
            "eid": "0011K00002H9qNmQAJ",
            "eobj": "Account",
            "eurl": null,
            "esys": "SALESFORCE",
            "dsp": true
          },
          {
            "id": "1P06GH6NC0YFN2YXBDUFZX2FIJADKKBZ3AD3",
            "base": false,
            "obj": "RelationshipType",
            "lbl": "Postman Team",
            "eid": null,
            "eobj": "RelationshipType",
            "eurl": null,
            "esys": "SALESFORCE",
            "dsp": false
          },
          {
            "id": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU",
            "base": true,
            "obj": "Relationship",
            "lbl": "7-Eleven - Ent 68639 - 10/1 - 68639",
            "eid": "a3o1K000001o9rKQAQ",
            "eobj": "Relationship",
            "eurl": null,
            "esys": "SALESFORCE",
            "dsp": true
          }
        ],
        "author": {
          "userId": null,
          "userName": null,
          "userEmail": null,
          "profile": null,
          "userType": null,
          "systemType": null,
          "id": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
          "name": "Sean Reed",
          "email": "sean.reed@postman.com",
          "companyId": null,
          "licenseType": null,
          "createdDate": null,
          "activeUser": false,
          "obj": null,
          "eid": "005Rf000002zZanIAE",
          "eobj": null,
          "eurl": null,
          "esys": null,
          "epp": null,
          "pp": null
        },
        "status": "EDITED",
        "tags": null,
        "attachments": [],
        "meta": {
          "color": null,
          "comments": 0,
          "likes": 0,
          "chars": 0,
          "size": 0,
          "edits": 0,
          "attachments": 0,
          "activityTypeId": "93ea9440-ad74-4011-957d-3236c1d084ce",
          "ctaId": null,
          "scoreMetaId": null,
          "systemGenerated": false,
          "source": "C360",
          "externalSource": null,
          "mediaUrl": null,
          "reportSynced": false,
          "sfdcEventId": null,
          "countOfSyncFailures": 0,
          "hasTask": false,
          "emailSent": false,
          "notesTemplateId": null,
          "activitySentiment": null,
          "genaiSentimentRationale": null,
          "genaiSentimentVerdictReasons": null,
          "genaiStatus": "NA",
          "externalSourceDetails": null,
          "comic": 0,
          "eid": null
        },
        "sfdcTaskId": null,
        "syncedToSFDC": false,
        "formMetadata": null,
        "mentions": [],
        "relatedRecords": null,
        "tasks": null,
        "associatedRecord": false,
        "gsAttachments": null,
        "new": false
      }
    ]
  },
  "result": true,
  "code": null,
  "message": null,
  "additionalInfo": {
    "unsupportedFilters": null,
    "highlightWords": []
  },
  "requestId": "34cc2a01-498e-4156-a521-a5d0f003c270"
}
```


---

## Reports & Analytics

### GET /v1/scorecards/config/reportV2
**URL:** `https://postman.us2.gainsightcloud.com/v1/scorecards/config/reportV2`
**Status Codes:** [200]
**Response Size:** 22122 - 22122 bytes

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "33ac9b74-99b9-4656-b5f5-6d5336ea9c1e",
  "data": {
    "scorecard": {
      "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ": {
        "entityType": "RELATIONSHIP",
        "exceptions": "{}",
        "groupRollup": true,
        "overallRollup": true,
        "relationshipTypeId": "1P06GH6NC0YFN2YXBDUFZX2FIJADKKBZ3AD3",
        "schemeType": null,
        "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
        "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
        "scorecardName": "Enterprise Scorecard",
        "measureMap": {
          "1I004WXB72YM0YG1FFFK38DF998IANVHV35F": {
            "gsid": "1I0013YW9ZK6KBW2CHC6SVT067OGLLPBFZEY",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F",
            "parentId": null,
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 1,
            "weight": 0.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": null,
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHC6SVT067OGLLPBFZEY",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "ROLLUP",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Emma Johnson-Prabhakar",
            "createdAt": 1737742604000,
            "modifiedAt": 1737742604000
          },
          "1I004WXB72YM0YG1FFIT4R9YKZ8L31UO0QT3": {
            "gsid": "1I0013YW9ZK6KBW2CHU3BFE1W3DR2IXUW3YQ",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FFIT4R9YKZ8L31UO0QT3",
            "parentId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 3,
            "weight": 40.0,
            "validityPeriod": 0,
            "validityPeriodType": "WEEKS",
            "description": "Calculates the percentage of seats provisioned by a team.",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHU3BFE1W3DR2IXUW3YQ",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Ben Wanless",
            "createdAt": 1737742604000,
            "modifiedAt": 1750801551000
          },
          "1I004WXB72YM0YG1FFEXIGGOE5WRRQ3HSC1B": {
            "gsid": "1I0013YW9ZK6KBW2CHWNX9HC11T006Q69D9Y",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FFEXIGGOE5WRRQ3HSC1B",
            "parentId": "1I004WXB72YM0YG1FFRFZOSV5UFBTEWCA356",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 1,
            "weight": 100.0,
            "validityPeriod": 0,
            "validityPeriodType": "WEEKS",
            "description": "Insights into how the customer has grown since joining Postman",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHWNX9HC11T006Q69D9Y",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Ben Wanless",
            "createdAt": 1737742604000,
            "modifiedAt": 1750801551000
          },
          "1I004WXB72YM0YG1FF8C9FKXTCB8HCHJU38R": {
            "gsid": "1I0013YW9ZK6KBW2CHK7LGOFHRPHMKG9TH4K",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FF8C9FKXTCB8HCHJU38R",
            "parentId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 1,
            "weight": 30.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHK7LGOFHRPHMKG9TH4K",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "GROUP",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Ben Wanless",
            "createdAt": 1737742604000,
            "modifiedAt": 1750801550000
          },
          "1I004WXB72YM0YG1FFRFZOSV5UFBTEWCA356": {
            "gsid": "1I0013YW9ZK6KBW2CHWGVC5ZEXZC1BHSJKB7",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FFRFZOSV5UFBTEWCA356",
            "parentId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 4,
            "weight": 0.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHWGVC5ZEXZC1BHSJKB7",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "GROUP",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Ben Wanless",
            "createdAt": 1737742604000,
            "modifiedAt": 1750801551000
          },
          "1I004WXB72YM0YG1FFTO7MH5CQBUZL72MBPY": {
            "gsid": "1I0013YW9ZK6KBW2CHMHNSG5O98XACFH1ORA",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FFTO7MH5CQBUZL72MBPY",
            "parentId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 3,
            "weight": 30.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHMHNSG5O98XACFH1ORA",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "GROUP",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Ben Wanless",
            "createdAt": 1737742604000,
            "modifiedAt": 1750801550000
          },
          "1I004WXB72YM0YG1FFJ6AMW57CNST64QBRIB": {
            "gsid": "1I0013YW9ZK6KBW2CH6JAYMWI0ATKJ20WZ0D",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FFJ6AMW57CNST64QBRIB",
            "parentId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 2,
            "weight": 30.0,
            "validityPeriod": 0,
            "validityPeriodType": "WEEKS",
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CH6JAYMWI0ATKJ20WZ0D",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Ben Wanless",
            "createdAt": 1737742604000,
            "modifiedAt": 1750801550000
          },
          "1I004WXB72YM0YG1FFLLX17FY74PSNQK66MN": {
            "gsid": "1I0013YW9ZK6KBW2CHGUIEUZFD6NGUFS416E",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FFLLX17FY74PSNQK66MN",
            "parentId": "1I004WXB72YM0YG1FF8C9FKXTCB8HCHJU38R",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 1,
            "weight": 100.0,
            "validityPeriod": 0,
            "validityPeriodType": "WEEKS",
            "description": "This factors in any open risk CTA's.",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHGUIEUZFD6NGUFS416E",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Ben Wanless",
            "createdAt": 1737742604000,
            "modifiedAt": 1750801550000
          },
          "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F": {
            "gsid": "1I0013YW9ZK6KBW2CHUVRO5D3WGJUJU7ZNKI",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "parentId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 2,
            "weight": 40.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHUVRO5D3WGJUJU7ZNKI",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "GROUP",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Ben Wanless",
            "createdAt": 1737742604000,
            "modifiedAt": 1750801551000
          },
          "1I004WXB72YM0YG1FFU0U8ITJ0BEI5K1TLJO": {
            "gsid": "1I0013YW9ZK6KBW2CHXM8FYMBBR7W1UFM60K",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FFU0U8ITJ0BEI5K1TLJO",
            "parentId": "1I004WXB72YM0YG1FFTO7MH5CQBUZL72MBPY",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 1,
            "weight": 100.0,
            "validityPeriod": 0,
            "validityPeriodType": "WEEKS",
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHXM8FYMBBR7W1UFM60K",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Ben Wanless",
            "createdAt": 1737742604000,
            "modifiedAt": 1750801551000
          },
          "1I004WXB72YM0YG1FFH73BJSSTYNWNTSPXZJ": {
            "gsid": "1I0013YW9ZK6KBW2CHYQPNWQ2GAD3V8X8DLH",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FFH73BJSSTYNWNTSPXZJ",
            "parentId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 1,
            "weight": 30.0,
            "validityPeriod": 0,
            "validityPeriodType": "WEEKS",
            "description": "Collaborative active user %",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHYQPNWQ2GAD3V8X8DLH",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Ben Wanless",
            "createdAt": 1737742604000,
            "modifiedAt": 1750801551000
          },
          "1I004WXB72YM0YG1FFT74EJHWAIN1GR7VQJH": {
            "gsid": "1I0013YW9ZK6KBW2CHQ5ESN1J0RJS65C2UFE",
            "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
            "measureId": "1I004WXB72YM0YG1FFT74EJHWAIN1GR7VQJH",
            "parentId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 4,
            "weight": 0.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": "{&quot;80-100&quot;:&quot;Nurture&quot;,&quot;50-80&quot;:&quot;Invest&quot;,&quot;0-50&quot;:&quot;Rescue&quot;}",
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHQ5ESN1J0RJS65C2UFE",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedById": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
            "createdBy": "Andor Fuhrer",
            "modifiedBy": "Ben Wanless",
            "createdAt": 1746096689000,
            "modifiedAt": 1750801550000
          }
        }
      },
      "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP": {
        "entityType": "RELATIONSHIP",
        "exceptions": "{}",
        "groupRollup": true,
        "overallRollup": true,
        "relationshipTypeId": "1P06GH6NC0YFN2YXBDUFZX2FIJADKKBZ3AD3",
        "schemeType": null,
        "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
        "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
        "scorecardName": "Hybrid/Digital",
        "measureMap": {
          "1I004WXB72YM0YG1FFGF1BF6IZ7B5RH39ME1": {
            "gsid": "1I0013YW9ZK6KBW2CHQF7HH9YUCTKJYKA6PI",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FFGF1BF6IZ7B5RH39ME1",
            "parentId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 5,
            "weight": 0.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHQF7HH9YUCTKJYKA6PI",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1742411830000,
            "modifiedAt": 1747073450000
          },
          "1I004WXB72YM0YG1FFHY47WVYLE15FBWBOCK": {
            "gsid": "1I0013YW9ZK6KBW2CH53HB275W7B8EDBWLV4",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FFHY47WVYLE15FBWBOCK",
            "parentId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 6,
            "weight": 0.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CH53HB275W7B8EDBWLV4",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1742411830000,
            "modifiedAt": 1747073450000
          },
          "1I004WXB72YM0YG1FFFK38DF998IANVHV35F": {
            "gsid": "1I0013YW9ZK6KBW2CHL4CA4Y3KBYXBQGGZ40",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F",
            "parentId": null,
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 1,
            "weight": 0.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": null,
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHL4CA4Y3KBYXBQGGZ40",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "ROLLUP",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Emma Johnson-Prabhakar",
            "createdAt": 1742411830000,
            "modifiedAt": 1742411830000
          },
          "1I004WXB72YM0YG1FF72K2MNXWN51JGO8QNF": {
            "gsid": "1I0013YW9ZK6KBW2CHMNLHHOPK4LMN36ID9T",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FF72K2MNXWN51JGO8QNF",
            "parentId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 4,
            "weight": 0.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "Checks whether the customer has SSO enabled",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHMNLHHOPK4LMN36ID9T",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1742411830000,
            "modifiedAt": 1747073450000
          },
          "1I004WXB72YM0YG1FFIT4R9YKZ8L31UO0QT3": {
            "gsid": "1I0013YW9ZK6KBW2CHMM8JO526AYC7ANW6TM",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FFIT4R9YKZ8L31UO0QT3",
            "parentId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 3,
            "weight": 40.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "Calculates the percentage of seats provisioned by a team.",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHMM8JO526AYC7ANW6TM",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1742411830000,
            "modifiedAt": 1747073450000
          },
          "1I004WXB72YM0YG1FFEXIGGOE5WRRQ3HSC1B": {
            "gsid": "1I0013YW9ZK6KBW2CHYYSV3CFT3JZFACURER",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FFEXIGGOE5WRRQ3HSC1B",
            "parentId": "1I004WXB72YM0YG1FFRFZOSV5UFBTEWCA356",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 1,
            "weight": 100.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "Insights into how the customer has grown since joining Postman",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHYYSV3CFT3JZFACURER",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1742411830000,
            "modifiedAt": 1747073450000
          },
          "1I004WXB72YM0YG1FF8C9FKXTCB8HCHJU38R": {
            "gsid": "1I0013YW9ZK6KBW2CH8EDH2QXHY17ZTCL13G",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FF8C9FKXTCB8HCHJU38R",
            "parentId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 1,
            "weight": 40.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CH8EDH2QXHY17ZTCL13G",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "GROUP",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1742411830000,
            "modifiedAt": 1747073450000
          },
          "1I004WXB72YM0YG1FFRFZOSV5UFBTEWCA356": {
            "gsid": "1I0013YW9ZK6KBW2CHCCVHOJUOGKPNWC5KGH",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FFRFZOSV5UFBTEWCA356",
            "parentId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 3,
            "weight": 0.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHCCVHOJUOGKPNWC5KGH",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "GROUP",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1742411830000,
            "modifiedAt": 1747073450000
          },
          "1I004WXB72YM0YG1FFJ6AMW57CNST64QBRIB": {
            "gsid": "1I0013YW9ZK6KBW2CHDRVKZM4KDS0GZC725H",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FFJ6AMW57CNST64QBRIB",
            "parentId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 2,
            "weight": 30.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHDRVKZM4KDS0GZC725H",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1742411830000,
            "modifiedAt": 1747073450000
          },
          "1I004WXB72YM0YG1FFLLX17FY74PSNQK66MN": {
            "gsid": "1I0013YW9ZK6KBW2CHPYO8OA9SU62SY9NORR",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FFLLX17FY74PSNQK66MN",
            "parentId": "1I004WXB72YM0YG1FF8C9FKXTCB8HCHJU38R",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 1,
            "weight": 100.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "This factors in any open risk CTA's.",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHPYO8OA9SU62SY9NORR",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1742411830000,
            "modifiedAt": 1747073450000
          },
          "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F": {
            "gsid": "1I0013YW9ZK6KBW2CHCP79TXZDI3U8KOHXB3",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "parentId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 2,
            "weight": 60.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CHCP79TXZDI3U8KOHXB3",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "GROUP",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1742411830000,
            "modifiedAt": 1747073450000
          },
          "1I004WXB72YM0YG1FFT74EJHWAIN1GR7VQJH": {
            "gsid": "1I0013YW9ZK6KBW2CH18PUBV7J1ZJVL9O2OI",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FFT74EJHWAIN1GR7VQJH",
            "parentId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 7,
            "weight": 0.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CH18PUBV7J1ZJVL9O2OI",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Andor Fuhrer",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1746096698000,
            "modifiedAt": 1747073450000
          },
          "1I004WXB72YM0YG1FFH73BJSSTYNWNTSPXZJ": {
            "gsid": "1I0013YW9ZK6KBW2CH2OS1MNQWF95AC5XEQ3",
            "scorecardId": "1I005U7MROXZTOJB5GJM5WG0Z1SQJT1Z7CFP",
            "measureId": "1I004WXB72YM0YG1FFH73BJSSTYNWNTSPXZJ",
            "parentId": "1I004WXB72YM0YG1FFPPHBFE749EMVWK8Y0F",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "active": true,
            "displayOrder": 1,
            "weight": 30.0,
            "validityPeriod": 0,
            "validityPeriodType": null,
            "description": "Collaborative active user %",
            "helpText": null,
            "type": "SCORECARD",
            "measureMapId": "1I0013YW9ZK6KBW2CH2OS1MNQWF95AC5XEQ3",
            "deleted": false,
            "name": null,
            "entityType": "RELATIONSHIP",
            "inputType": "CALCULATED",
            "levelType": "MEASURE",
            "children": null,
            "createdById": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "createdBy": "Emma Johnson-Prabhakar",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1742411830000,
            "modifiedAt": 1747073450000
          }
        }
      }
    }
  },
  "message": null,
  "localizedMessage": null
}
```


### GET /v3/bi/reporting-ui/bootstrap
**URL:** `https://postman.us2.gainsightcloud.com/v3/bi/reporting-ui/bootstrap`
**Status Codes:** [200]
**Response Size:** 19437 - 19437 bytes

**Response:**
```json
{
  "result": true,
  "requestId": "7e8e751e-1f83-4ea0-9512-f3ed608d992a",
  "data": {
    "IS_ENABLED": true,
    "REPORT_CONFIG_LIMITS": {
      "GRID": {
        "limit": 5000,
        "pageSizes": [
          50,
          100,
          200
        ],
        "ADDITIONAL_INLINE_EDITABLE_OBJECTS": [
          "case",
          "company",
          "relationship",
          "... [truncated]"
        ],
        "ADDITIONAL_ADD_RECORD_ENABLED_OBJECTS": []
      },
      "GAUGE": {
        "CONDITIONAL_COLORS_LIMIT": 6
      },
      "CHART": {
        "PREVIEW_CONFIG_IN_CODEPEN": false,
        "CHART_DATA_POINTS_LIMIT": 2000,
        "DATA_POINTS_LIMITS": {
          "BAR": 2000,
          "COLUMN": 2000,
          "COLUMN_LINE": 2000,
          "AREA": 2000,
          "DONUT": 500,
          "PIE": 500,
          "SCATTER": 2000,
          "STACKED_COLUMN": 2000,
          "STACKED_BAR": 2000,
          "HEATMAP": 2000,
          "BUBBLE": 500,
          "D3BUBBLE": 1000,
          "FUNNEL": 100
        }
      },
      "FIELDS": {
        "formula": 10,
        "show": 50,
        "groupBy": 10,
        "maxNestLevels": 2,
        "pageSize": 200,
        "GS_REPORT_READ_LIMIT": 1000,
        "group": 10
      },
      "FEATURE_ENABLEMENT": {
        "SFDC": {
          "MAX_NEST_LEVELS": 5,
          "RESTRICT_NULLS_IN_HAVING": [
            "CURRENCY"
          ],
          "RESTRICT_INCLUDE_NULLS_IN_HAVING": false,
          "EXTERNAL_SHARING": {
            "IMAGE": true,
            "EXCEL": true,
            "LINK": true,
            "PPT": true
          },
          "ENABLE_SCHEDULED_EXPORTS": true,
          "DISABLE_NULLS_POSITION": false,
          "STRICT_CHECK_FOR_EXPRESSION": true,
          "SERVER_SIDE_SORT": true,
          "SERVER_SIDE_SEARCH": true,
          "SERVER_SIDE_PAGINATION": false,
          "PIVOT": true,
          "FORMULA_FIELDS": false,
          "MAX_RECORD_LIMIT": 2000,
          "GS_REPORT_READ_LIMIT": 2000,
          "CHART_EDITOR": false,
          "FILTER_FIELD": false,
          "SUMMARIZE_BY_DATETIME": false,
          "COMPLEX_FISCAL_TIME_PERIOD_CONFIGURED": true,
          "ALLOW_MULTIPLE_FISCAL_SUMMARIZATIONS": false,
          "RESTRICTED_FIELDS_FOR_GROUP_BY": [
            "CURRENCY"
          ],
          "RESTRICTED_FIELDS_FOR_AGGREGATION": [
            "RICHTEXTAREA",
            "BOOLEAN"
          ],
          "DATATYPES_SORTED_IN_MEMORY": [
            "PICKLIST",
            "MULTISELECTDROPDOWNLIST"
          ],
          "RESTRICTED_FIELDS_FOR_SORTING": [],
          "RESTRICTED_SUMMARIZATIONS": [
            "FISCAL_MONTH",
            "FISCAL_WEEK"
          ],
          "MULTI_CURRENCY_FILTERS": {
            "WHERE": true,
            "HAVING": false
          }
        },
        "MONGO": {
          "MAX_NEST_LEVELS": 0,
          "RESTRICT_NULLS_IN_HAVING": [
            "ALL"
          ],
          "RESTRICT_INCLUDE_NULLS_IN_HAVING": true,
          "EXTERNAL_SHARING": {
            "IMAGE": true,
            "EXCEL": true,
            "LINK": true,
            "PPT": true
          },
          "ENABLE_SCHEDULED_EXPORTS": true,
          "DISABLE_NULLS_POSITION": true,
          "STRICT_CHECK_FOR_EXPRESSION": false,
          "SERVER_SIDE_SORT": true,
          "SERVER_SIDE_SEARCH": true,
          "SERVER_SIDE_PAGINATION": true,
          "PIVOT": true,
          "FORMULA_FIELDS": false,
          "MAX_RECORD_LIMIT": -1,
          "CHART_EDITOR": false,
          "FILTER_FIELD": false,
          "SUMMARIZE_BY_DATETIME": false,
          "COMPLEX_FISCAL_TIME_PERIOD_CONFIGURED": false,
          "ALLOW_MULTIPLE_FISCAL_SUMMARIZATIONS": false,
          "RESTRICTED_FIELDS_FOR_GROUP_BY": [
            "CURRENCY"
          ],
          "RESTRICTED_FIELDS_FOR_AGGREGATION": [
            "RICHTEXTAREA"
          ],
          "DATATYPES_SORTED_IN_MEMORY": [
            "PICKLIST",
            "MULTISELECTDROPDOWNLIST"
          ],
          "RESTRICTED_FIELDS_FOR_SORTING": [],
          "RESTRICTED_FIELDS_FOR_WHERE_FILTERING": [
            "MULTISELECTDROPDOWNLIST"
          ],
          "RESTRICTED_SUMMARIZATIONS": [
            "WEEK",
            "MONTH",
            "QUARTER",
            "... [truncated]"
          ],
          "MULTI_CURRENCY_FILTERS": {
            "WHERE": false,
            "HAVING": false
          }
        },
        "REDSHIFT": {
          "MAX_NEST_LEVELS": 2,
          "RESTRICT_NULLS_IN_HAVING": [],
          "RESTRICT_INCLUDE_NULLS_IN_HAVING": false,
          "EXTERNAL_SHARING": {
            "IMAGE": true,
            "EXCEL": true,
            "LINK": true,
            "PPT": true
          },
          "ENABLE_SCHEDULED_EXPORTS": true,
          "DISABLE_NULLS_POSITION": false,
          "STRICT_CHECK_FOR_EXPRESSION": false,
          "SERVER_SIDE_SORT": true,
          "SERVER_SIDE_SEARCH": true,
          "SERVER_SIDE_PAGINATION": true,
          "PIVOT": true,
          "FORMULA_FIELDS": true,
          "MAX_RECORD_LIMIT": -1,
          "CHART_EDITOR": false,
          "FILTER_FIELD": true,
          "SUMMARIZE_BY_DATETIME": false,
          "COMPLEX_FISCAL_TIME_PERIOD_CONFIGURED": false,
          "ALLOW_MULTIPLE_FISCAL_SUMMARIZATIONS": false,
          "RESTRICTED_FIELDS_FOR_GROUP_BY": [
            "CURRENCY"
          ],
          "RESTRICTED_FIELDS_FOR_AGGREGATION": [
            "RICHTEXTAREA"
          ],
          "DATATYPES_SORTED_IN_MEMORY": [
            "PICKLIST",
            "MULTISELECTDROPDOWNLIST"
          ],
          "RESTRICTED_FIELDS_FOR_SORTING": [],
          "RESTRICTED_SUMMARIZATIONS": [
            "FISCAL_WEEK",
            "FISCAL_MONTH"
          ],
          "MULTI_CURRENCY_FILTERS": {
            "WHERE": true,
            "HAVING": true
          }
        },
        "POSTGRES": {
          "MAX_NEST_LEVELS": 2,
          "RESTRICT_NULLS_IN_HAVING": [],
          "RESTRICT_INCLUDE_NULLS_IN_HAVING": false,
          "EXTERNAL_SHARING": {
            "IMAGE": true,
            "EXCEL": true,
            "LINK": true,
            "PPT": true
          },
          "ENABLE_SCHEDULED_EXPORTS": true,
          "DISABLE_NULLS_POSITION": false,
          "STRICT_CHECK_FOR_EXPRESSION": false,
          "SERVER_SIDE_SORT": true,
          "SERVER_SIDE_SEARCH": true,
          "SERVER_SIDE_PAGINATION": true,
          "PIVOT": true,
          "FORMULA_FIELDS": true,
          "MAX_RECORD_LIMIT": -1,
          "CHART_EDITOR": false,
          "SUMMARIZE_BY_DATETIME": false,
          "COMPLEX_FISCAL_TIME_PERIOD_CONFIGURED": false,
          "ALLOW_MULTIPLE_FISCAL_SUMMARIZATIONS": false,
          "RESTRICTED_FIELDS_FOR_GROUP_BY": [
            "CURRENCY"
          ],
          "RESTRICTED_FIELDS_FOR_AGGREGATION": [
            "RICHTEXTAREA"
          ],
          "DATATYPES_SORTED_IN_MEMORY": [
            "MULTISELECTDROPDOWNLIST"
          ],
          "RESTRICTED_FIELDS_FOR_SORTING": [],
          "RESTRICTED_SUMMARIZATIONS": [
            "FISCAL_WEEK",
            "FISCAL_MONTH"
          ],
          "MULTI_CURRENCY_FILTERS": {
            "WHERE": true,
            "HAVING": true
          }
        },
        "HAPOSTGRES": {
          "MAX_NEST_LEVELS": 2,
          "RESTRICT_NULLS_IN_HAVING": [],
          "RESTRICT_INCLUDE_NULLS_IN_HAVING": false,
          "EXTERNAL_SHARING": {
            "IMAGE": true,
            "EXCEL": true,
            "LINK": true,
            "PPT": true
          },
          "ENABLE_SCHEDULED_EXPORTS": true,
          "DISABLE_NULLS_POSITION": false,
          "STRICT_CHECK_FOR_EXPRESSION": false,
          "SERVER_SIDE_SORT": true,
          "SERVER_SIDE_SEARCH": true,
          "SERVER_SIDE_PAGINATION": true,
          "PIVOT": true,
          "FORMULA_FIELDS": true,
          "MAX_RECORD_LIMIT": -1,
          "CHART_EDITOR": false,
          "FILTER_FIELD": true,
          "SUMMARIZE_BY_DATETIME": false,
          "COMPLEX_FISCAL_TIME_PERIOD_CONFIGURED": false,
          "ALLOW_MULTIPLE_FISCAL_SUMMARIZATIONS": false,
          "RESTRICTED_FIELDS_FOR_GROUP_BY": [
            "CURRENCY"
          ],
          "RESTRICTED_FIELDS_FOR_AGGREGATION": [
            "RICHTEXTAREA"
          ],
          "DATATYPES_SORTED_IN_MEMORY": [
            "MULTISELECTDROPDOWNLIST"
          ],
          "RESTRICTED_FIELDS_FOR_SORTING": [],
          "RESTRICTED_SUMMARIZATIONS": [
            "FISCAL_WEEK",
            "FISCAL_MONTH"
          ],
          "MULTI_CURRENCY_FILTERS": {
            "WHERE": true,
            "HAVING": true
          }
        },
        "UNIVERSAL_DATA_SET": {
          "MAX_NEST_LEVELS": 2,
          "RESTRICT_NULLS_IN_HAVING": [],
          "RESTRICT_INCLUDE_NULLS_IN_HAVING": false,
          "EXTERNAL_SHARING": {
            "IMAGE": true,
            "EXCEL": true,
            "LINK": true,
            "PPT": true
          },
          "ENABLE_SCHEDULED_EXPORTS": true,
          "DISABLE_NULLS_POSITION": false,
          "STRICT_CHECK_FOR_EXPRESSION": false,
          "SERVER_SIDE_SORT": true,
          "SERVER_SIDE_SEARCH": true,
          "SERVER_SIDE_PAGINATION": true,
          "PIVOT": true,
          "FORMULA_FIELDS": true,
          "MAX_RECORD_LIMIT": -1,
          "CHART_EDITOR": false,
          "SUMMARIZE_BY_DATETIME": false,
          "FILTER_FIELD": true,
          "COMPLEX_FISCAL_TIME_PERIOD_CONFIGURED": false,
          "ALLOW_MULTIPLE_FISCAL_SUMMARIZATIONS": false,
          "RESTRICTED_FIELDS_FOR_GROUP_BY": [
            "CURRENCY"
          ],
          "RESTRICTED_FIELDS_FOR_AGGREGATION": [
            "RICHTEXTAREA"
          ],
          "DATATYPES_SORTED_IN_MEMORY": [
            "PICKLIST",
            "MULTISELECTDROPDOWNLIST"
          ],
          "RESTRICTED_FIELDS_FOR_SORTING": [],
          "RESTRICTED_SUMMARIZATIONS": [
            "FISCAL_WEEK",
            "FISCAL_MONTH"
          ],
          "MULTI_CURRENCY_FILTERS": {
            "WHERE": true,
            "HAVING": true
          }
        },
        "SNOWFLAKE": {
          "MAX_NEST_LEVELS": 2,
          "WEB_EXPORT_LIMIT": 5000,
          "RESTRICT_NULLS_IN_HAVING": [],
          "RESTRICT_INCLUDE_NULLS_IN_HAVING": false,
          "EXTERNAL_SHARING": {
            "IMAGE": true,
            "EXCEL": true,
            "LINK": true,
            "PPT": true
          },
          "ENABLE_SCHEDULED_EXPORTS": true,
          "DISABLE_NULLS_POSITION": false,
          "STRICT_CHECK_FOR_EXPRESSION": false,
          "SERVER_SIDE_SORT": true,
          "SERVER_SIDE_SEARCH": true,
          "SERVER_SIDE_PAGINATION": true,
          "PIVOT": true,
          "FORMULA_FIELDS": false,
          "MAX_RECORD_LIMIT": -1,
          "CHART_EDITOR": false,
          "FILTER_FIELD": true,
          "SUMMARIZE_BY_DATETIME": false,
          "COMPLEX_FISCAL_TIME_PERIOD_CONFIGURED": false,
          "ALLOW_MULTIPLE_FISCAL_SUMMARIZATIONS": true,
          "RESTRICTED_FIELDS_FOR_GROUP_BY": [
            "CURRENCY"
          ],
          "RESTRICTED_FIELDS_FOR_AGGREGATION": [
            "RICHTEXTAREA"
          ],
          "DATATYPES_SORTED_IN_MEMORY": [
            "MULTISELECTDROPDOWNLIST"
          ],
          "RESTRICTED_FIELDS_FOR_SORTING": [],
          "RESTRICTED_SUMMARIZATIONS": [
            "FISCAL_WEEK",
            "FISCAL_MONTH"
          ],
          "MULTI_CURRENCY_FILTERS": {
            "WHERE": true,
            "HAVING": true
          },
          "SOURCE_TYPE_PATH": "meta.properties.sourceType"
        },
        "DATABRICKS": {
          "MAX_NEST_LEVELS": 2,
          "RESTRICT_NULLS_IN_HAVING": [],
          "RESTRICT_INCLUDE_NULLS_IN_HAVING": false,
          "WEB_EXPORT_LIMIT": 5000,
          "EXTERNAL_SHARING": {
            "IMAGE": true,
            "EXCEL": true,
            "LINK": true,
            "PPT": true
          },
          "ENABLE_SCHEDULED_EXPORTS": true,
          "DISABLE_NULLS_POSITION": false,
          "STRICT_CHECK_FOR_EXPRESSION": false,
          "SERVER_SIDE_SORT": true,
          "SERVER_SIDE_SEARCH": true,
          "SERVER_SIDE_PAGINATION": true,
          "PIVOT": true,
          "FORMULA_FIELDS": true,
          "MAX_RECORD_LIMIT": -1,
          "CHART_EDITOR": false,
          "FILTER_FIELD": true,
          "SUMMARIZE_BY_DATETIME": false,
          "COMPLEX_FISCAL_TIME_PERIOD_CONFIGURED": false,
          "ALLOW_MULTIPLE_FISCAL_SUMMARIZATIONS": true,
          "RESTRICTED_FIELDS_FOR_GROUP_BY": [
            "CURRENCY"
          ],
          "RESTRICTED_FIELDS_FOR_AGGREGATION": [
            "RICHTEXTAREA"
          ],
          "DATATYPES_SORTED_IN_MEMORY": [
            "MULTISELECTDROPDOWNLIST"
          ],
          "RESTRICTED_FIELDS_FOR_SORTING": [],
          "RESTRICTED_SUMMARIZATIONS": [
            "FISCAL_WEEK",
            "FISCAL_MONTH"
          ],
          "MULTI_CURRENCY_FILTERS": {
            "WHERE": true,
            "HAVING": true
          },
          "SOURCE_TYPE_PATH": "meta.properties.sourceType"
        },
        "DATASPHERE": {
          "MAX_NEST_LEVELS": 2,
          "RESTRICT_NULLS_IN_HAVING": [],
          "RESTRICT_INCLUDE_NULLS_IN_HAVING": false,
          "EXTERNAL_SHARING": {
            "IMAGE": true,
            "EXCEL": true,
            "LINK": true,
            "PPT": true
          },
          "ENABLE_SCHEDULED_EXPORTS": true,
          "DISABLE_NULLS_POSITION": false,
          "STRICT_CHECK_FOR_EXPRESSION": false,
          "SERVER_SIDE_SORT": true,
          "SERVER_SIDE_SEARCH": true,
          "SERVER_SIDE_PAGINATION": true,
          "PIVOT": true,
          "FORMULA_FIELDS": true,
          "MAX_RECORD_LIMIT": -1,
          "CHART_EDITOR": false,
          "FILTER_FIELD": true,
          "SUMMARIZE_BY_DATETIME": false,
          "COMPLEX_FISCAL_TIME_PERIOD_CONFIGURED": false,
          "ALLOW_MULTIPLE_FISCAL_SUMMARIZATIONS": true,
          "RESTRICTED_FIELDS_FOR_GROUP_BY": [
            "CURRENCY"
          ],
          "RESTRICTED_FIELDS_FOR_AGGREGATION": [
            "RICHTEXTAREA"
          ],
          "DATATYPES_SORTED_IN_MEMORY": [
            "MULTISELECTDROPDOWNLIST"
          ],
          "RESTRICTED_FIELDS_FOR_SORTING": [],
          "RESTRICTED_SUMMARIZATIONS": [
            "FISCAL_WEEK",
            "FISCAL_MONTH"
          ],
          "MULTI_CURRENCY_FILTERS": {
            "WHERE": true,
            "HAVING": true
          },
          "SOURCE_TYPE_PATH": "meta.properties.sourceType"
        }
      },
      "UI_NON_RETRYABLE_ERROR_CODES": [
        "GS_BI_1055",
        "GS_BI_1061",
        "GS_BI_1057",
        "... [truncated]"
      ],
      "REPORT_NAME_LIMIT": {
        "contains": [
          "(Gs App Report)"
        ]
      }
    },
    "END_USER_REPORTING_CONFIGS": {
      "ACCESS_LEVEL_ACTIONS_MAPPING": {
        "READ": {
          "clonable": false,
          "movable": false,
          "share": false,
          "deletable": false,
          "markPrivate": false,
          "editable": false,
          "allowShareSettingChange": false,
          "managePermissions": false,
          "readable": true,
          "clearStatePreservationForAllUsers": false
        },
        "SHARE": {
          "clonable": false,
          "movable": false,
          "share": true,
          "deletable": false,
          "markPrivate": false,
          "editable": false,
          "allowShareSettingChange": false,
          "managePermissions": false,
          "readable": true,
          "clearStatePreservationForAllUsers": false
        },
        "WRITE": {
          "clonable": true,
          "movable": false,
          "share": false,
          "deletable": false,
          "markPrivate": false,
          "editable": true,
          "allowShareSettingChange": false,
          "managePermissions": false,
          "readable": true,
          "clearStatePreservationForAllUsers": false
        },
        "ADMIN": {
          "clonable": true,
          "movable": true,
          "share": true,
          "deletable": true,
          "markPrivate": true,
          "editable": true,
          "allowShareSettingChange": true,
          "managePermissions": true,
          "readable": true,
          "clearStatePreservationForAllUsers": true
        }
      },
      "REPORTING_CATEGORY_TYPE": [
        {
          "displayName": "All Reports",
          "value": "ALL_REPORTS",
          "applicableToAdmin": true,
          "applicableToEndUser": true
        },
        {
          "displayName": "My Reports",
          "value": "MY_REPORTS",
          "applicableToAdmin": true,
          "applicableToEndUser": true
        },
        {
          "displayName": "Shared With Me",
          "value": "SHARED_WITH_ME",
          "applicableToAdmin": false,
          "applicableToEndUser": true
        },
        "... [truncated]"
      ],
      "PERMISSIONS_CONFIG": {
        "PERMISSION_TYPES": [
          {
            "displayKey": "Private",
            "value": "private",
            "info": "info microcopy goes here",
            "adminOnly": false
          },
          {
            "displayKey": "Public - View",
            "value": "public-view",
            "info": "info microcopy goes here",
            "adminOnly": true
          },
          {
            "displayKey": "Public - View & Edit",
            "value": "public-view-edit",
            "info": "info microcopy goes here",
            "adminOnly": true
          },
          "... [truncated]"
        ],
        "PERMISSIONS_ACCESS_LEVELS": [
          {
            "displayKey": "View",
            "value": "view"
          },
          {
            "displayKey": "View & Edit",
            "value": "view-edit"
          },
          {
            "displayKey": "Admin",
            "value": "admin"
          }
        ]
      }
    },
    "EXPLORE_REPORTS_CONFIGS": {
      "IS_EXPLORE_REPORTS_ENABLED": true,
      "FILTERABLE_FIELDS": [
        "sourceObject"
      ],
      "LISTABLE_FIELDS": [
        "reportId",
        "description",
        "name"
      ]
    },
    "CHART": {
      "KPI": {
        "IS_KPI_PROGRESS_BAR_ENABLED": true,
        "IS_KPI_PAST_PERIOD_ENABLED": true,
        "MDA_PAST_PERIOD_FILTERS": [
          {
            "dateLiteral": "CUSTOM",
            "isApplicableForCustomFiscal": true
          },
          {
            "dateLiteral": "TODAY",
            "isApplicableForCustomFiscal": true
          },
          {
            "dateLiteral": "YESTERDAY",
            "isApplicableForCustomFiscal": true
          },
          "... [truncated]"
        ],
        "SFDC_PAST_PERIOD_FILTERS": [
          {
            "dateLiteral": "CUSTOM",
            "isApplicableForCustomFiscal": true
          },
          {
            "dateLiteral": "TODAY",
            "isApplicableForCustomFiscal": true
          },
          {
            "dateLiteral": "YESTERDAY",
            "isApplicableForCustomFiscal": true
          },
          "... [truncated]"
        ]
      },
      "PREVIEW_CONFIG_IN_CODEPEN": false,
      "ENABLE_MISSING_DATA_POINTS": true
    },
    "DEFAULT_AGGREGATIONS": {
      "BOOLEAN": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      },
      "NUMBER": {
        "aggregationFunction": "SUM",
        "outputDataType": "NUMBER"
      },
      "STRING": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      },
      "INTEGER": {
        "aggregationFunction": "SUM",
        "outputDataType": "NUMBER"
      },
      "DOUBLE": {
        "aggregationFunction": "SUM",
        "outputDataType": "NUMBER"
      },
      "LONG": {
        "aggregationFunction": "SUM",
        "outputDataType": "NUMBER"
      },
      "DATE": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      },
      "DATETIME": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      },
      "PICKLIST": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      },
      "CURRENCY": {
        "aggregationFunction": "SUM",
        "outputDataType": "CURRENCY"
      },
      "PERCENTAGE": {
        "aggregationFunction": "AVG",
        "outputDataType": "PERCENTAGE"
      },
      "PERCENT": {
        "aggregationFunction": "AVG",
        "outputDataType": "PERCENT"
      },
      "EMAIL": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      },
      "SFDCID": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      },
      "URL": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      },
      "GSID": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      },
      "MULTISELECTDROPDOWNLIST": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      },
      "LOOKUP": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      },
      "DEFAULT": {
        "aggregationFunction": "COUNT",
        "outputDataType": "NUMBER"
      }
    },
    "DEFAULT_SORT_ORDERS": {
      "BOOLEAN": "ASC",
      "CONTEXT": "ASC",
      "CURRENCY": "DESC",
      "DATE": "ASC",
      "DATETIME": "ASC",
      "DOUBLE": "DESC",
      "EMAIL": "ASC",
      "GSID": "ASC",
      "INTEGER": "DESC",
      "LONG": "DESC",
      "LOOKUP": "ASC",
      "MULTISELECTDROPDOWNLIST": "ASC",
      "NUMBER": "DESC",
      "PERCENT": "DESC",
      "PERCENTAGE": "DESC",
      "PICKLIST": "ASC",
      "SFDCID": "ASC",
      "STRING": "ASC",
      "TIMESTAMP": "ASC",
      "URL": "ASC",
      "WHATID": "ASC",
      "WHOID": "ASC"
    },
    "IS_GAINSIGHT_HOME_ENABLED": true,
    "REPORT_SETTINGS": {
      "ENABLE_ADD_RECORD": false,
      "ENABLE_NAMED_FIELDS_ADDITION": true,
      "ENABLE_REPORTING_INLINE_EDIT": true,
      "DISABLE_SHARED_DASHBOARD_EXPORTS": false,
      "ENABLE_DATA_CACHING": true,
      "CUSTOMIZE_DRILLDOWN_REPORT": true,
      "ENABLE_REPORT_WIDGET_ADD_RECORD": true,
      "ENABLE_REPORT_WIDGET_ADD_RECORD_CUSTOMISATION": false,
      "HONOR_SFDC_PERMISSIONS": false,
      "DISABLE_ALL_EXPORTS": false
    },
    "IS_END_USER_ENABLED": true,
    "IS_QUICK_INSIGHTS_REVAMP_ENABLED": true,
    "IS_PARTNER_USER": false,
    "IS_REPORTING_CHAT_ASSIST_ENABLED": false,
    "ENABLE_REPORT_LEVEL_COLORS": true,
    "IS_REPORTING_ADMIN": false,
    "IS_INLINE_EDIT_ENABLED": true,
    "LIST_FILTER_CONFIG": {
      "label": "",
      "children": [
        {
          "fieldName": "sourceObject",
          "label": "Source Object",
          "data": {
            "fieldName": "sourceObject",
            "label": "Source Object",
            "dataType": "PICKLIST",
            "options": []
          },
          "dataType": "PICKLIST",
          "icon": "multiselectdropdownlist",
          "children": []
        },
        {
          "fieldName": "createdBy",
          "label": "Created By",
          "data": {
            "fieldName": "createdBy",
            "label": "Created By",
            "dataType": "LOOKUP",
            "meta": {
              "properties": {
                "SEARCH_CONTROLLER": "AUTO_SUGGEST",
                "autoSuggestDetails": {
                  "object": "gsuser",
                  "searchOn": [
                    "Name",
                    "Email"
                  ],
                  "dataStore": "HAPOSTGRES",
                  "columnsToList": [
                    "Gsid"
                  ],
                  "connectionType": "MDA",
                  "connectionId": "MDA"
                },
                "sourceType": "GSID"
              }
            }
          },
          "source": "MDA",
          "dataStore": "HAPOSTGRES",
          "dataType": "LOOKUP",
          "icon": "lookup",
          "children": []
        },
        {
          "fieldName": "modifiedBy",
          "label": "Modified By",
          "data": {
            "fieldName": "modifiedBy",
            "label": "Modified By",
            "dataType": "LOOKUP",
            "meta": {
              "properties": {
                "SEARCH_CONTROLLER": "AUTO_SUGGEST",
                "autoSuggestDetails": {
                  "object": "gsuser",
                  "searchOn": [
                    "Name",
                    "Email"
                  ],
                  "dataStore": "HAPOSTGRES",
                  "columnsToList": [
                    "Gsid"
                  ],
                  "connectionType": "MDA",
                  "connectionId": "MDA"
                },
                "sourceType": "GSID"
              }
            }
          },
          "source": "MDA",
          "dataStore": "HAPOSTGRES",
          "dataType": "LOOKUP",
          "icon": "lookup",
          "children": []
        },
        "... [truncated]"
      ]
    },
    "FIELDS": [
      {
        "fieldName": "reportId",
        "label": "Report Id",
        "dataType": "STRING",
        "meta": {
          "hidden": true,
          "sortable": true
        }
      },
      {
        "fieldName": "reportTypes",
        "label": "Report Types",
        "dataType": "ARRAY",
        "meta": {
          "hidden": true,
          "sortable": false
        }
      },
      {
        "fieldName": "visualizationType",
        "label": "Visualization Type",
        "dataType": "STRING",
        "meta": {
          "hidden": true,
          "sortable": true
        }
      },
      "... [truncated]"
    ]
  },
  "duration": 65,
  "alerts": []
}
```


### POST /v1/scorecards/summary/customer360
**URL:** `https://postman.us2.gainsightcloud.com/v1/scorecards/summary/customer360`
**Status Codes:** [200]
**Response Size:** 58846 - 58846 bytes

**Request Body:**
```json
{
  "relationshipId": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU",
  "companyId": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
  "standardObjectType": "RELATIONSHIP"
}
```

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "6b85c0f6-ed6c-464d-8ea6-70d5a20ac9fc",
  "data": {
    "history": {
      "snapshots": [
        {
          "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
          "name": "Enterprise Scorecard",
          "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
          "snapshots": [
            {
              "snapshotDate": "2025-09-14",
              "createdDate": "2025-09-14T23:48:58.781+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-09-07",
              "createdDate": "2025-09-07T23:46:32.118+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-08-31",
              "createdDate": "2025-09-01T00:11:52.910+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-08-24",
              "createdDate": "2025-08-24T23:30:43.744+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-08-17",
              "createdDate": "2025-08-17T23:31:31.146+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-08-10",
              "createdDate": "2025-08-10T23:52:35.421+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-08-03",
              "createdDate": "2025-08-03T23:50:53.318+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-07-27",
              "createdDate": "2025-07-27T23:48:09.065+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-07-20",
              "createdDate": "2025-07-20T23:44:38.323+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-07-13",
              "createdDate": "2025-07-14T23:46:20.572+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-07-07",
              "createdDate": "2025-07-07T23:43:58.411+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-06-30",
              "createdDate": "2025-06-30T23:31:49.441+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-06-23",
              "createdDate": "2025-06-23T23:42:59.730+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-06-16",
              "createdDate": "2025-06-16T23:47:29.847+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-06-09",
              "createdDate": "2025-06-09T23:47:56.580+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-06-02",
              "createdDate": "2025-06-02T23:44:30.068+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-05-26",
              "createdDate": "2025-05-26T23:40:11.632+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            },
            {
              "snapshotDate": "2025-05-19",
              "createdDate": "2025-05-19T23:31:40.788+0000",
              "value": 65,
              "measureId": "1I004WXB72YM0YG1FFFK38DF998IANVHV35F"
            }
          ]
        }
      ]
    },
    "schemes": [
      {
        "id": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
        "gsid": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
        "name": "Numeric(0-100)",
        "type": "NUMERIC",
        "active": true,
        "rangeFrom": 0,
        "displayOrder": 1,
        "rangeTo": 100,
        "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
        "scoringSchemeDefinitionList": [
          {
            "id": "1I005J9UJ6741NB1T6I28VPSP24MVWSAYCNZ",
            "gsid": "1I005J9UJ6741NB1T6I28VPSP24MVWSAYCNZ",
            "name": "100",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 100.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390147,
            "modifiedAt": 1758014390147,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T68A5F78134I9CA2GA8U",
            "gsid": "1I005J9UJ6741NB1T68A5F78134I9CA2GA8U",
            "name": "99",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 99.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390147,
            "modifiedAt": 1758014390147,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6JC52HIYOAI19H296K9",
            "gsid": "1I005J9UJ6741NB1T6JC52HIYOAI19H296K9",
            "name": "98",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 98.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390147,
            "modifiedAt": 1758014390147,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6OT3HIJMM05JDXAGROY",
            "gsid": "1I005J9UJ6741NB1T6OT3HIJMM05JDXAGROY",
            "name": "97",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 97.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390147,
            "modifiedAt": 1758014390147,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6DYRQGP1EFWL383LBY1",
            "gsid": "1I005J9UJ6741NB1T6DYRQGP1EFWL383LBY1",
            "name": "96",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 96.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390147,
            "modifiedAt": 1758014390147,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6AR3FD61OATV9DOH14Z",
            "gsid": "1I005J9UJ6741NB1T6AR3FD61OATV9DOH14Z",
            "name": "95",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 95.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390147,
            "modifiedAt": 1758014390147,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6TY0HYPZVI56LYMXXC3",
            "gsid": "1I005J9UJ6741NB1T6TY0HYPZVI56LYMXXC3",
            "name": "94",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 94.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6WO9LIL7W6JNQ3RJFYN",
            "gsid": "1I005J9UJ6741NB1T6WO9LIL7W6JNQ3RJFYN",
            "name": "93",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 93.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T63R3WQHXZ3CHYBANN89",
            "gsid": "1I005J9UJ6741NB1T63R3WQHXZ3CHYBANN89",
            "name": "92",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 92.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T69V7N33L2ZQQ0LX7909",
            "gsid": "1I005J9UJ6741NB1T69V7N33L2ZQQ0LX7909",
            "name": "91",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 91.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6PYAS1IUVBRXVG45OJD",
            "gsid": "1I005J9UJ6741NB1T6PYAS1IUVBRXVG45OJD",
            "name": "90",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 90.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6Q6S8C06T02WOHGZW5R",
            "gsid": "1I005J9UJ6741NB1T6Q6S8C06T02WOHGZW5R",
            "name": "89",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 89.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6T1KHGLMWO6E6ABOHWW",
            "gsid": "1I005J9UJ6741NB1T6T1KHGLMWO6E6ABOHWW",
            "name": "88",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 88.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6VT4919K53VOTF9ODK0",
            "gsid": "1I005J9UJ6741NB1T6VT4919K53VOTF9ODK0",
            "name": "87",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 87.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6J00341AOS00L521LYX",
            "gsid": "1I005J9UJ6741NB1T6J00341AOS00L521LYX",
            "name": "86",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 86.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6PB19P03MX8AGAFNN3Y",
            "gsid": "1I005J9UJ6741NB1T6PB19P03MX8AGAFNN3Y",
            "name": "85",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 85.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6Z4E34NV2OT2Q2XPRHW",
            "gsid": "1I005J9UJ6741NB1T6Z4E34NV2OT2Q2XPRHW",
            "name": "84",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 84.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6MEC3Z9WQ2OGPSU0DZF",
            "gsid": "1I005J9UJ6741NB1T6MEC3Z9WQ2OGPSU0DZF",
            "name": "83",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 83.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390148,
            "modifiedAt": 1758014390148,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6ZKIGSN58UTI76VR27C",
            "gsid": "1I005J9UJ6741NB1T6ZKIGSN58UTI76VR27C",
            "name": "82",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 82.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6Q9RX6GM1RYA91OMXI3",
            "gsid": "1I005J9UJ6741NB1T6Q9RX6GM1RYA91OMXI3",
            "name": "81",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 81.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T628VWZQSS5S5GXSZ3ZD",
            "gsid": "1I005J9UJ6741NB1T628VWZQSS5S5GXSZ3ZD",
            "name": "80",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 80.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6Q97JG5RY1NQ71D9BWN",
            "gsid": "1I005J9UJ6741NB1T6Q97JG5RY1NQ71D9BWN",
            "name": "79",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 79.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T68QZMD92OWP4T1A6PJN",
            "gsid": "1I005J9UJ6741NB1T68QZMD92OWP4T1A6PJN",
            "name": "78",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 78.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T61P5KZ56TNRDVWARY2K",
            "gsid": "1I005J9UJ6741NB1T61P5KZ56TNRDVWARY2K",
            "name": "77",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 77.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6B6JQAIRXLXZOM0RZQC",
            "gsid": "1I005J9UJ6741NB1T6B6JQAIRXLXZOM0RZQC",
            "name": "76",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 75,
            "rangeTo": 100,
            "score": 76.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6AZAHL0VEWZDHVSSUT9",
            "gsid": "1I005J9UJ6741NB1T6AZAHL0VEWZDHVSSUT9",
            "name": "75",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 75.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T65NOD02ACEKYEP6C2Y9",
            "gsid": "1I005J9UJ6741NB1T65NOD02ACEKYEP6C2Y9",
            "name": "74",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 74.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6FF7HF3DV8VGYC4COF2",
            "gsid": "1I005J9UJ6741NB1T6FF7HF3DV8VGYC4COF2",
            "name": "73",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 73.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6ASCJP4872TY9SSKUNU",
            "gsid": "1I005J9UJ6741NB1T6ASCJP4872TY9SSKUNU",
            "name": "72",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 72.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6PFFFCPHZGMW28MHSAJ",
            "gsid": "1I005J9UJ6741NB1T6PFFFCPHZGMW28MHSAJ",
            "name": "71",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 71.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T63C5I1N0P5L09GNADPU",
            "gsid": "1I005J9UJ6741NB1T63C5I1N0P5L09GNADPU",
            "name": "70",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 70.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390149,
            "modifiedAt": 1758014390149,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6G7QXR7QEEFM9G5X6BA",
            "gsid": "1I005J9UJ6741NB1T6G7QXR7QEEFM9G5X6BA",
            "name": "69",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 69.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6ULQB605KO2L3R2EZ0P",
            "gsid": "1I005J9UJ6741NB1T6ULQB605KO2L3R2EZ0P",
            "name": "68",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 68.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6D3YEY2GCVNVMGFVTDD",
            "gsid": "1I005J9UJ6741NB1T6D3YEY2GCVNVMGFVTDD",
            "name": "67",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 67.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6KS195MOMFQE1KA0Y3P",
            "gsid": "1I005J9UJ6741NB1T6KS195MOMFQE1KA0Y3P",
            "name": "66",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 66.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6KOMVRRDGL2PC5GHSQC",
            "gsid": "1I005J9UJ6741NB1T6KOMVRRDGL2PC5GHSQC",
            "name": "65",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 65.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6ANACDRI5JQ53T5XX3W",
            "gsid": "1I005J9UJ6741NB1T6ANACDRI5JQ53T5XX3W",
            "name": "64",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 64.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6GJY7U0SJRQLZDOOM1R",
            "gsid": "1I005J9UJ6741NB1T6GJY7U0SJRQLZDOOM1R",
            "name": "63",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 63.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6MQ3I7OQHR9RF4TGY82",
            "gsid": "1I005J9UJ6741NB1T6MQ3I7OQHR9RF4TGY82",
            "name": "62",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 62.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6JPA6KI296COWWTW4Y8",
            "gsid": "1I005J9UJ6741NB1T6JPA6KI296COWWTW4Y8",
            "name": "61",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 61.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6MPEKC3Y3OZOGW824DW",
            "gsid": "1I005J9UJ6741NB1T6MPEKC3Y3OZOGW824DW",
            "name": "60",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 60.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6FWV8FAC7WS9T1H7H42",
            "gsid": "1I005J9UJ6741NB1T6FWV8FAC7WS9T1H7H42",
            "name": "59",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 59.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T61SGFVDZHKOG5OTA54Q",
            "gsid": "1I005J9UJ6741NB1T61SGFVDZHKOG5OTA54Q",
            "name": "58",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 58.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6P9W0TC0XA7RF11Z17E",
            "gsid": "1I005J9UJ6741NB1T6P9W0TC0XA7RF11Z17E",
            "name": "57",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 57.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390150,
            "modifiedAt": 1758014390150,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T68PA9C4L91E31MSI2Y4",
            "gsid": "1I005J9UJ6741NB1T68PA9C4L91E31MSI2Y4",
            "name": "56",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 56.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6NXT3UL3GGHBMDFIK4S",
            "gsid": "1I005J9UJ6741NB1T6NXT3UL3GGHBMDFIK4S",
            "name": "55",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 55.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6D4DMB49PJHKQ9H6G7R",
            "gsid": "1I005J9UJ6741NB1T6D4DMB49PJHKQ9H6G7R",
            "name": "54",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 54.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6FP39NGYRM18WB8EI1A",
            "gsid": "1I005J9UJ6741NB1T6FP39NGYRM18WB8EI1A",
            "name": "53",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 53.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6BNP7LZNU91OG0ME7M9",
            "gsid": "1I005J9UJ6741NB1T6BNP7LZNU91OG0ME7M9",
            "name": "52",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 52.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6JX4KLJQHV8DE632Y9R",
            "gsid": "1I005J9UJ6741NB1T6JX4KLJQHV8DE632Y9R",
            "name": "51",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 75,
            "score": 51.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6QOL60XF6HVGLNCOIJ2",
            "gsid": "1I005J9UJ6741NB1T6QOL60XF6HVGLNCOIJ2",
            "name": "50",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 50.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6FBCSY3DI7SGQARCTWM",
            "gsid": "1I005J9UJ6741NB1T6FBCSY3DI7SGQARCTWM",
            "name": "49",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 49.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6GANS2IREP6AIA0DYKK",
            "gsid": "1I005J9UJ6741NB1T6GANS2IREP6AIA0DYKK",
            "name": "48",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 48.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6N93NPP9ESDIITU2181",
            "gsid": "1I005J9UJ6741NB1T6N93NPP9ESDIITU2181",
            "name": "47",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 47.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6A4B7MQ4R3ZDRFMYJW3",
            "gsid": "1I005J9UJ6741NB1T6A4B7MQ4R3ZDRFMYJW3",
            "name": "46",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 46.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6G4PCQCZG1BPO7NJX34",
            "gsid": "1I005J9UJ6741NB1T6G4PCQCZG1BPO7NJX34",
            "name": "45",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 45.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6JBRP65YJLZI8WHYSMH",
            "gsid": "1I005J9UJ6741NB1T6JBRP65YJLZI8WHYSMH",
            "name": "44",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 44.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390151,
            "modifiedAt": 1758014390151,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T63758650IKQQT4X296Z",
            "gsid": "1I005J9UJ6741NB1T63758650IKQQT4X296Z",
            "name": "43",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 43.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6LWZE2XFUH2R3HRJBUJ",
            "gsid": "1I005J9UJ6741NB1T6LWZE2XFUH2R3HRJBUJ",
            "name": "42",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 42.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T648X9XK996DW4HS9EVH",
            "gsid": "1I005J9UJ6741NB1T648X9XK996DW4HS9EVH",
            "name": "41",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 41.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6J7CYU9WCC2KDUQ8F8B",
            "gsid": "1I005J9UJ6741NB1T6J7CYU9WCC2KDUQ8F8B",
            "name": "40",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 40.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6G9U74ZSD74HHS8D50B",
            "gsid": "1I005J9UJ6741NB1T6G9U74ZSD74HHS8D50B",
            "name": "39",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 39.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6HORKHIBCJPJ1PU8M6Y",
            "gsid": "1I005J9UJ6741NB1T6HORKHIBCJPJ1PU8M6Y",
            "name": "38",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 38.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6SM80GILEMSIU2TZK0A",
            "gsid": "1I005J9UJ6741NB1T6SM80GILEMSIU2TZK0A",
            "name": "37",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 37.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6MBDAFQ6X05XF973BSU",
            "gsid": "1I005J9UJ6741NB1T6MBDAFQ6X05XF973BSU",
            "name": "36",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 36.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T62D6GO60KAPT6Z8LWVN",
            "gsid": "1I005J9UJ6741NB1T62D6GO60KAPT6Z8LWVN",
            "name": "35",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 35.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6IPBEM06IPYP1QGU344",
            "gsid": "1I005J9UJ6741NB1T6IPBEM06IPYP1QGU344",
            "name": "34",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 34.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T65ST2Y9ZZV4908WRIK3",
            "gsid": "1I005J9UJ6741NB1T65ST2Y9ZZV4908WRIK3",
            "name": "33",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 33.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6J0YMK82GVC0XVG3ZDH",
            "gsid": "1I005J9UJ6741NB1T6J0YMK82GVC0XVG3ZDH",
            "name": "32",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 32.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T62BQFUYXUDNMFJBX4UF",
            "gsid": "1I005J9UJ6741NB1T62BQFUYXUDNMFJBX4UF",
            "name": "31",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 31.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390152,
            "modifiedAt": 1758014390152,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6UV4NQGLSAGFTKJARBI",
            "gsid": "1I005J9UJ6741NB1T6UV4NQGLSAGFTKJARBI",
            "name": "30",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 30.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6F0NA1PWFS1WJPWE0GM",
            "gsid": "1I005J9UJ6741NB1T6F0NA1PWFS1WJPWE0GM",
            "name": "29",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 29.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6SPIUYKP9DUI1604FXT",
            "gsid": "1I005J9UJ6741NB1T6SPIUYKP9DUI1604FXT",
            "name": "28",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 28.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6A0NG0C6SRA9A6P22ZX",
            "gsid": "1I005J9UJ6741NB1T6A0NG0C6SRA9A6P22ZX",
            "name": "27",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 27.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6S0KP2ER9XY0DS3GB31",
            "gsid": "1I005J9UJ6741NB1T6S0KP2ER9XY0DS3GB31",
            "name": "26",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 26.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T61HV71KIJJ9B8ZG6CQT",
            "gsid": "1I005J9UJ6741NB1T61HV71KIJJ9B8ZG6CQT",
            "name": "25",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 25.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T69GJUE07R01XAD0FRXY",
            "gsid": "1I005J9UJ6741NB1T69GJUE07R01XAD0FRXY",
            "name": "24",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 24.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6EA1MVHG110YRC3JHQ2",
            "gsid": "1I005J9UJ6741NB1T6EA1MVHG110YRC3JHQ2",
            "name": "23",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 23.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T60S0KOQTSW8XJ6QK1OY",
            "gsid": "1I005J9UJ6741NB1T60S0KOQTSW8XJ6QK1OY",
            "name": "22",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 22.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T61ZP4AVA70GWZG6EIR3",
            "gsid": "1I005J9UJ6741NB1T61ZP4AVA70GWZG6EIR3",
            "name": "21",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 21.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6VAST3BN2SUOBE7UQG9",
            "gsid": "1I005J9UJ6741NB1T6VAST3BN2SUOBE7UQG9",
            "name": "20",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 20.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6MAHLF08934G3DYYOM1",
            "gsid": "1I005J9UJ6741NB1T6MAHLF08934G3DYYOM1",
            "name": "19",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 19.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6HMSKIZBLXBUKPCKUR0",
            "gsid": "1I005J9UJ6741NB1T6HMSKIZBLXBUKPCKUR0",
            "name": "18",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 18.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390153,
            "modifiedAt": 1758014390153,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6PXVKR7ZHDS7I5UOAF2",
            "gsid": "1I005J9UJ6741NB1T6PXVKR7ZHDS7I5UOAF2",
            "name": "17",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 17.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T60UVSIXIHF0QMCKU1RU",
            "gsid": "1I005J9UJ6741NB1T60UVSIXIHF0QMCKU1RU",
            "name": "16",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 16.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T66WEMG6JJYJHO9WZW4H",
            "gsid": "1I005J9UJ6741NB1T66WEMG6JJYJHO9WZW4H",
            "name": "15",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 15.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T69EPJRAL7MZRQCXCGIE",
            "gsid": "1I005J9UJ6741NB1T69EPJRAL7MZRQCXCGIE",
            "name": "14",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 14.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6QK2XJRBHUFVZKCDQWJ",
            "gsid": "1I005J9UJ6741NB1T6QK2XJRBHUFVZKCDQWJ",
            "name": "13",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 13.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6LA8DHFGHGOLSPQYXKT",
            "gsid": "1I005J9UJ6741NB1T6LA8DHFGHGOLSPQYXKT",
            "name": "12",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 12.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T60L42547Z1LQJR78AM6",
            "gsid": "1I005J9UJ6741NB1T60L42547Z1LQJR78AM6",
            "name": "11",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 11.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6EURVKN1FCNC1YV5GVO",
            "gsid": "1I005J9UJ6741NB1T6EURVKN1FCNC1YV5GVO",
            "name": "10",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 10.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6CUYJJSISY8W9HKNKWV",
            "gsid": "1I005J9UJ6741NB1T6CUYJJSISY8W9HKNKWV",
            "name": "9",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 9.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6ZZED4OXTRKFNVSOKWS",
            "gsid": "1I005J9UJ6741NB1T6ZZED4OXTRKFNVSOKWS",
            "name": "8",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 8.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6MROBHANKJGU9WC2OQP",
            "gsid": "1I005J9UJ6741NB1T6MROBHANKJGU9WC2OQP",
            "name": "7",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 7.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T66O46HN29MQVLNFIZED",
            "gsid": "1I005J9UJ6741NB1T66O46HN29MQVLNFIZED",
            "name": "6",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 6.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6L3FTX9PRO29JY9VX9Z",
            "gsid": "1I005J9UJ6741NB1T6L3FTX9PRO29JY9VX9Z",
            "name": "5",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 5.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390154,
            "modifiedAt": 1758014390154,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6M3UOK080OKJU6T5V5Q",
            "gsid": "1I005J9UJ6741NB1T6M3UOK080OKJU6T5V5Q",
            "name": "4",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 4.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390155,
            "modifiedAt": 1758014390155,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6JVFE713GONP717V9ZP",
            "gsid": "1I005J9UJ6741NB1T6JVFE713GONP717V9ZP",
            "name": "3",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 3.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390155,
            "modifiedAt": 1758014390155,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T64ZW5ZP1Y6EIQ4H1ZZI",
            "gsid": "1I005J9UJ6741NB1T64ZW5ZP1Y6EIQ4H1ZZI",
            "name": "2",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 2.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390155,
            "modifiedAt": 1758014390155,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6YMG63BSP0IS41D0FHX",
            "gsid": "1I005J9UJ6741NB1T6YMG63BSP0IS41D0FHX",
            "name": "1",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 1.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390155,
            "modifiedAt": 1758014390155,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6XNMINRPBJT19XFX7I4",
            "gsid": "1I005J9UJ6741NB1T6XNMINRPBJT19XFX7I4",
            "name": "0",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDPT79W6FG9X7071Q88C",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 0.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390155,
            "modifiedAt": 1758014390155,
            "deleted": false
          }
        ],
        "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
        "createdAt": 1758014390068,
        "modifiedAt": 1758014390068,
        "deleted": false,
        "definitionChanged": false
      },
      {
        "id": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
        "gsid": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
        "name": "Color(RYG)",
        "type": "COLOR",
        "active": true,
        "rangeFrom": 0,
        "displayOrder": 2,
        "rangeTo": 100,
        "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
        "scoringSchemeDefinitionList": [
          {
            "id": "1I005J9UJ6741NB1T6FBXLFSB8TCQP3ZISGC",
            "gsid": "1I005J9UJ6741NB1T6FBXLFSB8TCQP3ZISGC",
            "name": "Healthy",
            "label": "Healthy",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "color": "#5ab08e",
            "active": false,
            "rangeFrom": 80,
            "rangeTo": 100,
            "score": 90.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390157,
            "modifiedAt": 1758014390157,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6BHM0JD2XXDTX3AONZ3",
            "gsid": "1I005J9UJ6741NB1T6BHM0JD2XXDTX3AONZ3",
            "name": "Monitor",
            "label": "Monitor",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "color": "#e0a85f",
            "active": false,
            "rangeFrom": 50,
            "rangeTo": 80,
            "score": 65.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390157,
            "modifiedAt": 1758014390157,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6OUWIR7B95GAR9DP0SY",
            "gsid": "1I005J9UJ6741NB1T6OUWIR7B95GAR9DP0SY",
            "name": "At-Risk",
            "label": "At-Risk",
            "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
            "color": "#d76e75",
            "active": false,
            "rangeFrom": 0,
            "rangeTo": 50,
            "score": 25.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "modifiedById": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
            "modifiedBy": "Andor Fuhrer",
            "createdAt": 1758014390157,
            "modifiedAt": 1758014390157,
            "deleted": false
          }
        ],
        "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
        "createdAt": 1758014390068,
        "modifiedAt": 1758014390068,
        "deleted": false,
        "definitionChanged": false
      },
      {
        "id": "1I00RBG0CSO4TDK8RDT9HWUOXXMD4HS3SABE",
        "gsid": "1I00RBG0CSO4TDK8RDT9HWUOXXMD4HS3SABE",
        "name": "Grade(A-F)",
        "type": "GRADE",
        "active": true,
        "rangeFrom": 0,
        "displayOrder": 3,
        "rangeTo": 100,
        "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
        "scoringSchemeDefinitionList": [
          {
            "id": "1I005J9UJ6741NB1T6SJMAD1JFWUETAVI9NF",
            "gsid": "1I005J9UJ6741NB1T6SJMAD1JFWUETAVI9NF",
            "name": "A",
            "label": "A",
            "schemeId": "1I00RBG0CSO4TDK8RDT9HWUOXXMD4HS3SABE",
            "color": "#5ab08e",
            "active": true,
            "rangeFrom": 90,
            "rangeTo": 100,
            "score": 95.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "createdAt": 1758014390158,
            "modifiedAt": 1758014390158,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6H254Z044SBXC9OREZS",
            "gsid": "1I005J9UJ6741NB1T6H254Z044SBXC9OREZS",
            "name": "B",
            "label": "B",
            "schemeId": "1I00RBG0CSO4TDK8RDT9HWUOXXMD4HS3SABE",
            "color": "#3fce96",
            "active": true,
            "rangeFrom": 80,
            "rangeTo": 90,
            "score": 85.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "createdAt": 1758014390159,
            "modifiedAt": 1758014390159,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T60EPJDNOAWWZWNOT2AN",
            "gsid": "1I005J9UJ6741NB1T60EPJDNOAWWZWNOT2AN",
            "name": "C",
            "label": "C",
            "schemeId": "1I00RBG0CSO4TDK8RDT9HWUOXXMD4HS3SABE",
            "color": "#e0a85f",
            "active": true,
            "rangeFrom": 60,
            "rangeTo": 80,
            "score": 70.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "createdAt": 1758014390159,
            "modifiedAt": 1758014390159,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6UZL920J79BQ405R3D5",
            "gsid": "1I005J9UJ6741NB1T6UZL920J79BQ405R3D5",
            "name": "D",
            "label": "D",
            "schemeId": "1I00RBG0CSO4TDK8RDT9HWUOXXMD4HS3SABE",
            "color": "#fcc642",
            "active": true,
            "rangeFrom": 50,
            "rangeTo": 60,
            "score": 55.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "createdAt": 1758014390159,
            "modifiedAt": 1758014390159,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6AFXLOC6ZRO974G4XIZ",
            "gsid": "1I005J9UJ6741NB1T6AFXLOC6ZRO974G4XIZ",
            "name": "E",
            "label": "E",
            "schemeId": "1I00RBG0CSO4TDK8RDT9HWUOXXMD4HS3SABE",
            "color": "#f77271",
            "active": true,
            "rangeFrom": 40,
            "rangeTo": 50,
            "score": 45.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "createdAt": 1758014390159,
            "modifiedAt": 1758014390159,
            "deleted": false
          },
          {
            "id": "1I005J9UJ6741NB1T6BQWVF31H47Y8UU8XID",
            "gsid": "1I005J9UJ6741NB1T6BQWVF31H47Y8UU8XID",
            "name": "F",
            "label": "F",
            "schemeId": "1I00RBG0CSO4TDK8RDT9HWUOXXMD4HS3SABE",
            "color": "#d76e75",
            "active": true,
            "rangeFrom": 0,
            "rangeTo": 40,
            "score": 20.0,
            "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
            "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
            "createdAt": 1758014390159,
            "modifiedAt": 1758014390159,
            "deleted": false
          }
        ],
        "createdById": "1P01Q1IO46XBY7CLHNGTPHPME9Q1L4IZEM8F",
        "createdAt": 1758014390068,
        "modifiedAt": 1758014390068,
        "deleted": false,
        "definitionChanged": false
      }
    ],
    "factData": {
      "scorecardId": "1I005U7MROXZTOJB5GQ5CUEEAPMYP2O9EHYQ",
      "schemeId": "1I00RBG0CSO4TDK8RDLKX6AG9DGCAEMTH3HK",
      "scores": [
        {
          "measureName": "Overall Score (Relationship)",
          "measureLevel": "ROLLUP",
          "scoreId": "1I005J9UJ6741NB1T6BHM0JD2XXDTX3AONZ3",
          "value": 65,
          "color": "#e0a85f",
          "label": "Monitor",
          "trend": 0,
          "lastUpdatedOn": 1758000554552
        }
      ]
    }
  },
  "message": null,
  "localizedMessage": null
}
```


### POST /v3/bi/query/fetch-data-count
**URL:** `https://postman.us2.gainsightcloud.com/v3/bi/query/fetch-data-count?connectionType=MDA`
**Query Parameters:** connectionType
**Status Codes:** [200]
**Response Size:** 113 - 113 bytes

**Request Body:**
```json
{
  "reportId": "83b5e7f7-87de-47f3-bb94-218ddf041dbb",
  "reportName": "API Creates WoW",
  "reportDescription": "",
  "sourceDetails": {
    "objectName": "product_usage_team_metrics_weekly__gc",
    "objectLabel": "Product Usage Team Metrics Weekly",
    "connectionId": "MDA",
    "connectionType": "MDA",
    "dataStoreType": "HAPOSTGRES"
  },
  "showFields": [
    {
      "fieldName": "Metric_Value__gc",
      "dbName": "metric_value__gc",
      "label": "Metric Value",
      "fieldAlias": "sum_of_product_usage_team_metrics_weekly__gc_Metric_Value__gc",
      "dataType": "number",
      "objectName": "product_usage_team_metrics_weekly__gc",
      "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
      "properties": {
        "required": false,
        "aggregatable": true,
        "pathLabel": "Product Usage Team Metrics Weekly [Metric Value]",
        "columnCalculationConfigs": [],
        "inlineEditable": false
      },
      "displayOrder": 4,
      "connectionId": "MDA",
      "connectionType": "MDA",
      "fieldType": "calculated",
      "displayName": "SUM of Metric Value",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true,
      "expressionDetails": {
        "expressionType": "aggregationFunction",
        "expression": {
          "tokenType": "aggregationFunction",
          "key": "SUM",
          "outputDataType": "NUMBER"
        }
      }
    }
  ],
  "drillDownFields": [
    {
      "fieldName": "Name",
      "dbName": "gsd73342",
      "label": "Name",
      "fieldAlias": "relationship_Postman_Team_ID__gr_Name",
      "dataType": "string",
      "objectName": "relationship",
      "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
      "key": "Postman_Team_ID__gc_Name",
      "properties": {
        "required": true,
        "pathLabel": "Postman Team ID"
      },
      "fieldPath": {
        "lookupId": "Postman_Team_ID__gr",
        "legacyLookupId": "fcb50f21-f7f8-4290-b408-c23dc63093fc",
        "lookupName": "Postman_Team_ID__gr",
        "left": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "relationship",
          "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        },
        "right": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "product_usage_team_metrics_weekly__gc",
          "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        }
      },
      "connectionId": "MDA",
      "connectionType": "MDA",
      "fieldType": "field",
      "displayName": "Postman Team Name",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    },
    {
      "fieldName": "Week_Timestamp__gc",
      "dbName": "week_timestamp__gc",
      "label": "Week Timestamp",
      "fieldAlias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
      "dataType": "date",
      "objectName": "product_usage_team_metrics_weekly__gc",
      "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
      "key": "Week_Timestamp__gc",
      "properties": {
        "required": false,
        "aggregatable": true,
        "pathLabel": "Product Usage Team Metrics Weekly [Week Timestamp]",
        "columnCalculationConfigs": []
      },
      "displayOrder": 1,
      "connectionId": "MDA",
      "connectionType": "MDA",
      "fieldType": "field",
      "displayName": "Week Timestamp",
      "rowGrouped": false,
      "pivoted": false,
      "orderByInfo": {
        "nulls": "LAST",
        "order": "ASC"
      },
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    },
    {
      "fieldName": "Metric_Name__gc",
      "dbName": "metric_name__gc",
      "label": "Metric Name",
      "fieldAlias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
      "dataType": "string",
      "objectName": "product_usage_team_metrics_weekly__gc",
      "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
      "key": "Metric_Name__gc",
      "properties": {
        "required": false,
        "aggregatable": true,
        "pathLabel": "Product Usage Team Metrics Weekly [Metric Name]",
        "columnCalculationConfigs": []
      },
      "displayOrder": 2,
      "connectionId": "MDA",
      "connectionType": "MDA",
      "fieldType": "field",
      "displayName": "Metric Name",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    },
    {
      "fieldName": "Metric_Value__gc",
      "dbName": "metric_value__gc",
      "label": "Metric Value",
      "fieldAlias": "product_usage_team_metrics_weekly__gc_Metric_Value__gc",
      "dataType": "number",
      "objectName": "product_usage_team_metrics_weekly__gc",
      "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
      "key": "Metric_Value__gc",
      "properties": {
        "required": false,
        "aggregatable": true,
        "pathLabel": "Product Usage Team Metrics Weekly [Metric Value]",
        "columnCalculationConfigs": []
      },
      "displayOrder": 1,
      "connectionId": "MDA",
      "connectionType": "MDA",
      "fieldType": "field",
      "displayName": "Metric Value",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    },
    {
      "fieldName": "User_Count__gc",
      "dbName": "user_count__gc",
      "label": "User Count",
      "fieldAlias": "product_usage_team_metrics_weekly__gc_User_Count__gc",
      "dataType": "number",
      "objectName": "product_usage_team_metrics_weekly__gc",
      "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
      "key": "User_Count__gc",
      "properties": {
        "required": false,
        "pathLabel": "Product Usage Team Metrics Weekly [User Count]"
      },
      "connectionId": "MDA",
      "connectionType": "MDA",
      "displayName": "User Count",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    },
    {
      "fieldName": "Enterprise_Age__gc",
      "dbName": "enterprise_age__gc",
      "label": "Enterprise Age",
      "fieldAlias": "relationship_Postman_Team_ID__gr_Enterprise_Age__gc",
      "dataType": "number",
      "objectName": "relationship",
      "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
      "key": "Postman_Team_ID__gc_Enterprise_Age__gc",
      "properties": {
        "required": false,
        "pathLabel": "Postman Team ID \u2192 Enterprise Age"
      },
      "fieldPath": {
        "lookupId": "Postman_Team_ID__gr",
        "legacyLookupId": "fcb50f21-f7f8-4290-b408-c23dc63093fc",
        "lookupName": "Postman_Team_ID__gr",
        "left": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "relationship",
          "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        },
        "right": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "product_usage_team_metrics_weekly__gc",
          "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        }
      },
      "connectionId": "MDA",
      "connectionType": "MDA",
      "displayName": "Postman Team Enterprise Age",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    },
    {
      "fieldName": "Renewal_Date__gc",
      "dbName": "renewal_date__gc",
      "label": "Renewal Date",
      "fieldAlias": "relationship_Postman_Team_ID__gr_Renewal_Date__gc",
      "dataType": "date",
      "objectName": "relationship",
      "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
      "key": "Postman_Team_ID__gc_Renewal_Date__gc",
      "properties": {
        "required": false,
        "pathLabel": "Postman Team ID \u2192 Renewal Date"
      },
      "fieldPath": {
        "lookupId": "Postman_Team_ID__gr",
        "legacyLookupId": "fcb50f21-f7f8-4290-b408-c23dc63093fc",
        "lookupName": "Postman_Team_ID__gr",
        "left": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "relationship",
          "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        },
        "right": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "product_usage_team_metrics_weekly__gc",
          "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        }
      },
      "connectionId": "MDA",
      "connectionType": "MDA",
      "displayName": "Postman Team Renewal Date",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    },
    {
      "fieldName": "Name",
      "dbName": "gsd73733",
      "label": "Name",
      "fieldAlias": "company_CompanyId__gr_Postman_Team_ID__gr_Name",
      "dataType": "string",
      "objectName": "company",
      "objectDBName": "company_3a5744bf41214e2eab6fee7bf36fc524",
      "key": "Postman_Team_ID__gc_CompanyId_Name",
      "properties": {
        "required": true,
        "pathLabel": "Postman Team ID \u2192 Company Id"
      },
      "fieldPath": {
        "lookupId": "Postman_Team_ID__gr",
        "legacyLookupId": "fcb50f21-f7f8-4290-b408-c23dc63093fc",
        "lookupName": "Postman_Team_ID__gr",
        "left": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "relationship",
          "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        },
        "right": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "product_usage_team_metrics_weekly__gc",
          "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        },
        "fieldPath": {
          "lookupId": "CompanyId__gr",
          "legacyLookupId": "873d2b46-a974-4c88-a892-82ea0017b4aa",
          "lookupName": "CompanyId__gr",
          "left": {
            "type": "BASE_FIELD",
            "fieldName": "Gsid",
            "dbName": "gsid",
            "label": "GSID",
            "objectName": "company",
            "objectDBName": "company_3a5744bf41214e2eab6fee7bf36fc524",
            "hasLookup": false,
            "displayOrder": 0
          },
          "right": {
            "type": "BASE_FIELD",
            "fieldName": "CompanyId",
            "dbName": "gsd3569",
            "label": "Company Id",
            "objectName": "relationship",
            "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
            "hasLookup": false,
            "displayOrder": 0
          }
        }
      },
      "connectionId": "MDA",
      "connectionType": "MDA",
      "fieldType": "field",
      "displayName": "Company Name",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    },
    {
      "fieldName": "Postman_Team_ARR__gc",
      "dbName": "postman_team_arr__gc",
      "label": "Postman Team ARR",
      "fieldAlias": "relationship_Postman_Team_ID__gr_Postman_Team_ARR__gc",
      "dataType": "currency",
      "objectName": "relationship",
      "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
      "key": "Postman_Team_ID__gc_Postman_Team_ARR__gc",
      "properties": {
        "required": false,
        "pathLabel": "Postman Team ID \u2192 Postman Team ARR"
      },
      "fieldPath": {
        "lookupId": "Postman_Team_ID__gr",
        "legacyLookupId": "fcb50f21-f7f8-4290-b408-c23dc63093fc",
        "lookupName": "Postman_Team_ID__gr",
        "left": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "relationship",
          "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        },
        "right": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "product_usage_team_metrics_weekly__gc",
          "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        }
      },
      "connectionId": "MDA",
      "connectionType": "MDA",
      "displayName": "Postman Team Postman Team ARR",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    },
    {
      "fieldName": "Name",
      "dbName": "gsd14784",
      "label": "Name",
      "fieldAlias": "gsuser_Csm__gr_Postman_Team_ID__gr_Name",
      "dataType": "string",
      "objectName": "gsuser",
      "objectDBName": "user_3a5744bf41214e2eab6fee7bf36fc524",
      "key": "Postman_Team_ID__gc_Csm_Name",
      "properties": {
        "SEARCH_CONTROLLER": "AUTO_SUGGEST",
        "required": false,
        "pathLabel": "Postman Team ID \u2192 CSM \u2192 Name"
      },
      "fieldPath": {
        "lookupId": "Postman_Team_ID__gr",
        "legacyLookupId": "fcb50f21-f7f8-4290-b408-c23dc63093fc",
        "lookupName": "Postman_Team_ID__gr",
        "left": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "relationship",
          "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        },
        "right": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "objectName": "product_usage_team_metrics_weekly__gc",
          "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
          "hasLookup": false,
          "displayOrder": 0
        },
        "fieldPath": {
          "lookupId": "Csm__gr",
          "legacyLookupId": "f3541dcc-61bf-4046-8352-9c51218d3206",
          "lookupName": "Csm__gr",
          "left": {
            "type": "BASE_FIELD",
            "fieldName": "Gsid",
            "dbName": "gsid",
            "label": "GSID",
            "objectName": "gsuser",
            "objectDBName": "user_3a5744bf41214e2eab6fee7bf36fc524",
            "hasLookup": false,
            "displayOrder": 0
          },
          "right": {
            "type": "BASE_FIELD",
            "fieldName": "Csm",
            "dbName": "gsd2607",
            "label": "CSM",
            "objectName": "relationship",
            "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
            "hasLookup": false,
            "displayOrder": 0
          }
        }
      },
      "connectionId": "MDA",
      "connectionType": "MDA",
      "displayName": "CSM Name",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    }
  ],
  "groupByFields": [
    {
      "fieldName": "Week_Timestamp__gc",
      "dbName": "week_timestamp__gc",
      "label": "Week Timestamp",
      "fieldAlias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
      "dataType": "date",
      "objectName": "product_usage_team_metrics_weekly__gc",
      "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
      "properties": {
        "required": false,
        "aggregatable": true,
        "pathLabel": "Product Usage Team Metrics Weekly [Week Timestamp]",
        "columnCalculationConfigs": []
      },
      "displayOrder": 1,
      "connectionId": "MDA",
      "connectionType": "MDA",
      "fieldType": "field",
      "displayName": "Week Timestamp",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    },
    {
      "fieldName": "Metric_Name__gc",
      "dbName": "metric_name__gc",
      "label": "Metric Name",
      "fieldAlias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
      "dataType": "string",
      "objectName": "product_usage_team_metrics_weekly__gc",
      "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
      "properties": {
        "required": false,
        "aggregatable": true,
        "pathLabel": "Product Usage Team Metrics Weekly [Metric Name]",
        "columnCalculationConfigs": [],
        "alwaysIncludeInColorMaster": true,
        "colorDisabled": false
      },
      "displayOrder": 2,
      "connectionId": "MDA",
      "connectionType": "MDA",
      "fieldType": "field",
      "displayName": "Metric Name",
      "rowGrouped": false,
      "pivoted": false,
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true,
      "expressionDetails": {}
    }
  ],
  "orderByFields": [
    {
      "fieldName": "Week_Timestamp__gc",
      "dbName": "week_timestamp__gc",
      "label": "Week Timestamp",
      "fieldAlias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
      "dataType": "date",
      "objectName": "product_usage_team_metrics_weekly__gc",
      "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
      "properties": {
        "required": false,
        "aggregatable": true,
        "pathLabel": "Product Usage Team Metrics Weekly [Week Timestamp]",
        "columnCalculationConfigs": []
      },
      "displayOrder": 1,
      "connectionId": "MDA",
      "connectionType": "MDA",
      "fieldType": "field",
      "displayName": "Week Timestamp",
      "rowGrouped": false,
      "pivoted": false,
      "orderByInfo": {
        "nulls": "LAST",
        "order": "ASC"
      },
      "scale": 0,
      "groupable": true,
      "sortable": true,
      "filterable": true
    }
  ],
  "whereFilters": {
    "conditions": [
      {
        "locked": true,
        "filterAlias": "A",
        "comparisonOperator": "CONTAINS",
        "rightOperandType": "VALUE",
        "filterValue": {
          "value": [
            "API creates"
          ]
        },
        "includeNulls": false,
        "includeEmptyValues": false,
        "anyAtGlobal": false,
        "global": false,
        "leftOperand": {
          "fieldName": "Metric_Name__gc",
          "dbName": "metric_name__gc",
          "label": "Metric Name",
          "dataType": "STRING",
          "objectName": "product_usage_team_metrics_weekly__gc",
          "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Product Usage Team Metrics Weekly",
          "key": "Metric_Name__gc",
          "properties": {
            "sourceType": "STRING",
            "pathLabel": "Metric Name"
          },
          "fieldType": "field",
          "rowGrouped": false,
          "pivoted": false,
          "groupable": false,
          "sortable": false,
          "filterable": false
        }
      }
    ],
    "expression": "A"
  },
  "havingFilters": {
    "conditions": [],
    "expression": ""
  },
  "pageSize": 50,
  "reportDisplayType": "LINE",
  "reportOptions": {
    "normalize": false,
    "cumulative": false,
    "enableMilestone": false,
    "enableStacking": false,
    "enableCumulativePercentage": false,
    "showLabels": true,
    "enableRGBColor": true,
    "enableDualYAxis": true,
    "enableTextWrap": false,
    "freezeFirstColumn": false,
    "enableRanking": false,
    "enableSmoothLine": false,
    "enableMissingDatapoints": false,
    "enableClientSideRowGrouping": false,
    "enableConditionalColoring": false,
    "enableAddRecord": false,
    "enableDataExport": true,
    "rowHeight": "S",
    "enableComments": false
  },
  "reportType": "adhoc",
  "reportTypes": [],
  "properties": {
    "creatable": true
  },
  "additionalGlobalFilters": {
    "conditions": [
      {
        "filterAlias": "G1",
        "logicalOperator": "AND",
        "comparisonOperator": "EQ",
        "rightOperandType": "VALUE",
        "filterValue": {
          "value": [
            "68639"
          ]
        },
        "anyAtGlobal": false,
        "global": false,
        "leftOperand": {
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "dataType": "LOOKUP",
          "objectName": "product_usage_team_metrics_weekly__gc",
          "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Product Usage Team Metrics Weekly",
          "fieldType": "Field",
          "rowGrouped": false,
          "pivoted": false,
          "groupable": false,
          "sortable": false,
          "filterable": false
        }
      }
    ],
    "expression": "G1"
  }
}
```

**Response:**
```json
{
  "result": true,
  "requestId": "d9120657-2839-4dd9-91fc-fbc8c3c3d8a0",
  "data": {
    "count": 45
  },
  "duration": 355,
  "alerts": []
}
```


### POST /v3/bi/reporting/section/report-data
**URL:** `https://postman.us2.gainsightcloud.com/v3/bi/reporting/section/report-data?useCache=false&requestSource=R360&includeId=true&piedc=true&entityId=rId&c360RevampEnabled=true`
**Query Parameters:** useCache, requestSource, includeId, piedc, entityId, c360RevampEnabled
**Status Codes:** [200]
**Response Size:** 41077 - 41077 bytes

**Request Body:**
```json
{
  "reportId": "83b5e7f7-87de-47f3-bb94-218ddf041dbb",
  "userId": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
  "widgetId": "g_5958b914-3286-410f-8111-c99adb0cf6fb",
  "sourceDetails": {},
  "properties": {},
  "sectionId": "84441ebc-079b-4e3d-85f0-d0010b5a3eed",
  "widgetGlobalFilters": {
    "conditions": [
      {
        "leftOperand": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "dataType": "LOOKUP",
          "objectName": "product_usage_team_metrics_weekly__gc",
          "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Product Usage Team Metrics Weekly",
          "key": "Postman_Team_ID__gc"
        },
        "filterAlias": "bm_0",
        "logicalOperator": "AND",
        "comparisonOperator": "EQ",
        "rightOperandType": "VALUE",
        "filterValue": {
          "value": [
            "68639"
          ]
        }
      }
    ],
    "expression": "bm_0"
  }
}
```

**Response:**
```json
{
  "result": true,
  "requestId": "9f03bb3e-b168-41e0-a100-01627befb04a",
  "data": {
    "data": {
      "configuration": {
        "plotOptions": {
          "series": {
            "states": {
              "inactive": {
                "opacity": 1
              }
            },
            "boostThreshold": 100,
            "allowPointSelect": false,
            "animation": {
              "duration": 0
            },
            "cursor": "pointer",
            "shadow": false,
            "compare": "",
            "point": {
              "events": null
            },
            "events": null,
            "marker": {
              "enabled": true,
              "radius": 4
            },
            "dataLabels": {
              "enabled": false,
              "overflow": "justify",
              "color": "#19232F",
              "format": "{point.label}",
              "allowOverlap": false,
              "style": {
                "fontFamily": "Noto Sans, Noto Sans JP, Sans-serif",
                "textOutline": "none",
                "fontWeight": "normal",
                "textShadow": ""
              }
            },
            "visible": true,
            "turboThreshold": 5000
          },
          "bubble": {
            "animationLimit": 20
          },
          "packedbubble": {
            "pointArrayMap": [
              "value"
            ],
            "pointValKey": "value",
            "minSize": "25%",
            "maxSize": "100%",
            "layoutAlgorithm": {
              "splitSeries": false,
              "enableSimulation": false
            },
            "dataLabels": {
              "enabled": true,
              "format": "{point.name}",
              "filter": {
                "property": "value",
                "operator": ">=",
                "value": 1
              }
            }
          },
          "bar": {
            "borderRadius": 2
          },
          "column": {
            "borderRadius": 2
          },
          "flags": {
            "tooltip": {
              "pointFormat": "<div class='box'><div class='series-name'><div class='point-name'><b>{point.label} On:</b><span class='value'>&nbsp;{point.name}</span></div><div class='label'><b>{point.text}</b></div><div class='lab... [truncated]"
            }
          },
          "pie": null,
          "scatter": null,
          "heatmap": null,
          "area": null,
          "line": null,
          "funnel": null,
          "solidgauge": {
            "dataLabels": {
              "y": -30,
              "borderWidth": 0,
              "useHTML": true,
              "format": "<div style='cursor: pointer'><div class='guage-point-label'>{point.label}</div><div class='guage-point-name'>{point.name}</div></div>"
            },
            "rounded": true
          }
        },
        "xAxis": {
          "allowDecimals": false,
          "stackLabels": {
            "enabled": false,
            "style": {
              "fontWeight": "normal",
              "color": "#19232F",
              "fontFamily": "Noto Sans, Noto Sans JP, Sans-serif",
              "textShadow": ""
            }
          },
          "title": {
            "text": "",
            "style": {
              "color": "#8399AF",
              "fontFamily": "Noto Sans, Noto Sans JP, \"Proxima Nova Rg\", Sans-serif"
            }
          },
          "labels": {
            "rotation": 0,
            "style": {
              "fontSize": "12px",
              "color": "#8399AF",
              "fontFamily": "Noto Sans, Noto Sans JP, \"Proxima Nova Rg\", Sans-serif"
            },
            "enabled": true
          },
          "minPadding": 0,
          "categories": [
            "1/1/2024",
            "1/8/2024",
            "2/5/2024",
            "... [truncated]"
          ]
        },
        "legend": {
          "maxHeight": 100,
          "itemMarginTop": 5,
          "itemMarginBottom": 5,
          "itemStyle": {
            "color": "#8399AF",
            "fontFamily": "Noto Sans, Noto Sans JP, Sans-serif",
            "fontSize": "14px"
          },
          "enabled": true
        },
        "tooltip": {
          "backgroundColor": "#FFFFFF",
          "style": {
            "color": "#374351",
            "fontFamily": "Noto Sans, Noto Sans JP, Sans-serif"
          },
          "borderWidth": 0,
          "shadow": false,
          "borderRadius": 0,
          "borderColor": "#dbdbdb",
          "useHTML": true,
          "headerFormat": "",
          "pointFormat": "<div class='box'><div class='point-name'>{point.name}</div><div class='series-name'><div class='label'><b>{series.name}</b>&nbsp;<span class='value'>{point.label}</span></div></div></div>"
        },
        "title": {
          "text": "",
          "style": {
            "fontWeight": "normal",
            "fontSize": "16px",
            "fontFamily": "Noto Sans, Noto Sans JP, Sans-serif"
          }
        },
        "yAxis": [
          {
            "allowDecimals": false,
            "stackLabels": {
              "enabled": false,
              "style": {
                "fontWeight": "normal",
                "color": "#19232F",
                "fontFamily": "Noto Sans, Noto Sans JP, Sans-serif",
                "textShadow": ""
              }
            },
            "title": {
              "text": "SUM of Metric Value",
              "style": {
                "color": "#8399AF",
                "fontFamily": "Noto Sans, Noto Sans JP, \"Proxima Nova Rg\", Sans-serif"
              }
            },
            "labels": {
              "rotation": 0,
              "style": {
                "fontSize": "12px",
                "color": "#8399AF",
                "fontFamily": "Noto Sans, Noto Sans JP, \"Proxima Nova Rg\", Sans-serif"
              },
              "enabled": true
            },
            "minPadding": 0,
            "min": 0
          }
        ],
        "credits": {
          "enabled": false
        },
        "series": [],
        "boost": {
          "enabled": false
        },
        "pane": {
          "center": [
            "50%",
            "50%"
          ],
          "size": "90%",
          "startAngle": -130,
          "endAngle": 130,
          "background": {
            "backgroundColor": "#F0F4F7",
            "borderWidth": 0,
            "innerRadius": "85%",
            "outerRadius": "100%",
            "shape": "arc"
          }
        },
        "chart": {
          "zoomType": "xy",
          "alignTicks": false,
          "resetZoomButton": {
            "theme": {
              "fill": "white",
              "stroke": "silver",
              "r": 5,
              "states": {
                "hover": {
                  "fill": "#41739D",
                  "style": {
                    "color": "white"
                  }
                }
              }
            }
          },
          "type": "line"
        }
      },
      "options": {
        "lang": {
          "decimalPoint": ".",
          "thousandsSep": ","
        }
      },
      "data": [
        {
          "data": [
            {
              "x": 0,
              "y": 1.0,
              "z": "1",
              "pct": "100%",
              "name": "1/1/2024",
              "label": "1",
              "pointId": "b782adf6-9113-43c7-826a-e105a33da576"
            },
            {
              "x": 1,
              "y": 3.0,
              "z": "5",
              "pct": "60%",
              "name": "1/8/2024",
              "label": "3",
              "pointId": "b78fe2dd-af31-46b2-8840-319d6849b7a0"
            },
            {
              "x": 2,
              "y": 1.0,
              "z": "1",
              "pct": "100%",
              "name": "2/5/2024",
              "label": "1",
              "pointId": "b2c08223-bace-4b6d-832f-37f2af5a7f09"
            },
            {
              "x": 3,
              "y": 1.0,
              "z": "1",
              "pct": "100%",
              "name": "2/12/2024",
              "label": "1",
              "pointId": "4448f672-ea9f-4209-8a6f-43c925d05f54"
            },
            {
              "x": 4,
              "y": 1.0,
              "z": "1",
              "pct": "100%",
              "name": "3/4/2024",
              "label": "1",
              "pointId": "f352bd73-4a75-458a-8828-5717a4f6c8ba"
            },
            {
              "x": 5,
              "y": 14.0,
              "z": "28",
              "pct": "50%",
              "name": "3/11/2024",
              "label": "14",
              "pointId": "0e8a0e72-ed0b-49a4-95a3-519e55f92d0c"
            },
            {
              "x": 6,
              "y": 4.0,
              "z": "4",
              "pct": "100%",
              "name": "3/18/2024",
              "label": "4",
              "pointId": "ee7a4e42-eb34-4e94-9acd-cd9257a8610c"
            },
            {
              "x": 7,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "4/15/2024",
              "label": "1",
              "pointId": "e1ed3747-071b-4d65-9f44-d58a0f70b308"
            },
            {
              "x": 8,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "4/22/2024",
              "label": "1",
              "pointId": "c32bbafa-94ca-4e19-bbc9-61cab532029c"
            },
            {
              "x": 9,
              "y": 3.0,
              "z": "6",
              "pct": "50%",
              "name": "5/20/2024",
              "label": "3",
              "pointId": "e492591d-b8eb-4b60-bbab-4301c4f7a5d1"
            },
            {
              "x": 10,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "6/10/2024",
              "label": "1",
              "pointId": "74245773-e21f-4ef6-9ba2-a2e0a1c4fcbe"
            },
            {
              "x": 11,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "6/17/2024",
              "label": "1",
              "pointId": "82e4c3f9-28d3-4960-8dfb-15a6d0dcca93"
            },
            {
              "x": 12,
              "y": 1.0,
              "z": "1",
              "pct": "100%",
              "name": "7/8/2024",
              "label": "1",
              "pointId": "aa57293a-2791-48f4-b09f-7a59c6227139"
            },
            {
              "x": 13,
              "y": 2.0,
              "z": "3",
              "pct": "67%",
              "name": "8/5/2024",
              "label": "2",
              "pointId": "5d0c89ab-7889-4d71-848d-c96cc37c7663"
            },
            {
              "x": 14,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "8/19/2024",
              "label": "1",
              "pointId": "a913a048-9ac1-4f2a-b30a-aa880e6b5c24"
            },
            {
              "x": 15,
              "y": 2.0,
              "z": "2",
              "pct": "100%",
              "name": "9/16/2024",
              "label": "2",
              "pointId": "a6883f2a-59db-40fb-a1a5-a061f0bbcef5"
            },
            {
              "x": 16,
              "y": 2.0,
              "z": "3",
              "pct": "67%",
              "name": "9/23/2024",
              "label": "2",
              "pointId": "e9cda0d2-81c6-47ff-ba67-922ab881538a"
            },
            {
              "x": 17,
              "y": 2.0,
              "z": "3",
              "pct": "67%",
              "name": "10/14/2024",
              "label": "2",
              "pointId": "9917c8dc-4e61-4c3c-8964-ebf4695b4b3d"
            },
            {
              "x": 18,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "11/18/2024",
              "label": "1",
              "pointId": "93cd6baf-aa7e-4906-943f-abd368777472"
            },
            {
              "x": 19,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "2/10/2025",
              "label": "1",
              "pointId": "93de8cbf-c040-428e-99a2-e438e3c6ba25"
            },
            {
              "x": 20,
              "y": 1.0,
              "z": "1",
              "pct": "100%",
              "name": "2/17/2025",
              "label": "1",
              "pointId": "c01e8f55-b969-40b0-bff7-6ba9071464dd"
            },
            {
              "x": 21,
              "y": 1.0,
              "z": "1",
              "pct": "100%",
              "name": "2/24/2025",
              "label": "1",
              "pointId": "2fb6450a-2ea1-46f2-9839-c18645230788"
            },
            {
              "x": 22,
              "y": 2.0,
              "z": "2",
              "pct": "100%",
              "name": "3/10/2025",
              "label": "2",
              "pointId": "f9058eec-e7df-4078-bd14-d5a44f167cd9"
            },
            {
              "x": 23,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "3/24/2025",
              "label": "1",
              "pointId": "d9b71218-395f-42cd-b5c8-6a0412625f5b"
            },
            {
              "x": 24,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "7/7/2025",
              "label": "1",
              "pointId": "f1ba80bb-f61d-4d04-b4d9-7e30b4c6e224"
            },
            {
              "x": 25,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "7/14/2025",
              "label": "1",
              "pointId": "95190881-be17-4a54-8ad9-f2cb3286d48b"
            },
            {
              "x": 26,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "8/4/2025",
              "label": "1",
              "pointId": "97f4af66-f937-494a-aa7c-a2e13d3b7321"
            },
            {
              "x": 27,
              "y": 1.0,
              "z": "1",
              "pct": "100%",
              "name": "9/8/2025",
              "label": "1",
              "pointId": "94789e6c-944c-4976-930e-3f14bb07d7f0"
            }
          ],
          "color": "rgba(65, 177, 238, 1)",
          "name": "API creates",
          "index": 1,
          "legendIndex": 0
        },
        {
          "data": [
            {
              "x": 0,
              "y": 0,
              "z": "__NULL__",
              "pct": "__NULL__",
              "name": "1/1/2024",
              "dataLabels": {
                "enabled": false
              }
            },
            {
              "x": 1,
              "y": 2.0,
              "z": "5",
              "pct": "40%",
              "name": "1/8/2024",
              "label": "2",
              "pointId": "b3720b8b-ae75-4ced-8b03-0249f2f5356e"
            },
            {
              "x": 2,
              "y": 0,
              "z": "__NULL__",
              "pct": "__NULL__",
              "name": "2/5/2024",
              "dataLabels": {
                "enabled": false
              }
            },
            {
              "x": 3,
              "y": 0,
              "z": "__NULL__",
              "pct": "__NULL__",
              "name": "2/12/2024",
              "dataLabels": {
                "enabled": false
              }
            },
            {
              "x": 4,
              "y": 0,
              "z": "__NULL__",
              "pct": "__NULL__",
              "name": "3/4/2024",
              "dataLabels": {
                "enabled": false
              }
            },
            {
              "x": 5,
              "y": 14.0,
              "z": "28",
              "pct": "50%",
              "name": "3/11/2024",
              "label": "14",
              "pointId": "d35ccc70-aa52-4313-b9fd-8c907db179fa"
            },
            {
              "x": 6,
              "y": 0,
              "z": "__NULL__",
              "pct": "__NULL__",
              "name": "3/18/2024",
              "dataLabels": {
                "enabled": false
              }
            },
            {
              "x": 7,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "4/15/2024",
              "label": "1",
              "pointId": "d80d5bc3-70e8-4bce-ab14-acbfcdacbfb5"
            },
            {
              "x": 8,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "4/22/2024",
              "label": "1",
              "pointId": "71f6ac50-1da8-4db0-8018-edb4f6d40ddd"
            },
            {
              "x": 9,
              "y": 3.0,
              "z": "6",
              "pct": "50%",
              "name": "5/20/2024",
              "label": "3",
              "pointId": "78e75a21-275a-4433-b89b-35c3ba28fc75"
            },
            {
              "x": 10,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "6/10/2024",
              "label": "1",
              "pointId": "8b637272-5570-46cc-9293-c2968f52227d"
            },
            {
              "x": 11,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "6/17/2024",
              "label": "1",
              "pointId": "7f2e4b28-3823-490f-ba13-6e1099d0e936"
            },
            {
              "x": 12,
              "y": 0,
              "z": "__NULL__",
              "pct": "__NULL__",
              "name": "7/8/2024",
              "dataLabels": {
                "enabled": false
              }
            },
            {
              "x": 13,
              "y": 1.0,
              "z": "3",
              "pct": "33%",
              "name": "8/5/2024",
              "label": "1",
              "pointId": "def87560-4782-44ee-a0b7-f91a771f7657"
            },
            {
              "x": 14,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "8/19/2024",
              "label": "1",
              "pointId": "9a32820b-0743-4dba-90f0-9f14385f795c"
            },
            {
              "x": 15,
              "y": 0,
              "z": "__NULL__",
              "pct": "__NULL__",
              "name": "9/16/2024",
              "dataLabels": {
                "enabled": false
              }
            },
            {
              "x": 16,
              "y": 1.0,
              "z": "3",
              "pct": "33%",
              "name": "9/23/2024",
              "label": "1",
              "pointId": "60a5c917-26e0-4759-90ac-993a3798a99d"
            },
            {
              "x": 17,
              "y": 1.0,
              "z": "3",
              "pct": "33%",
              "name": "10/14/2024",
              "label": "1",
              "pointId": "292b2f1a-f869-4d83-b8bd-4fb268072407"
            },
            {
              "x": 18,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "11/18/2024",
              "label": "1",
              "pointId": "910f1d36-fbf6-48bf-be60-b3734aa43e50"
            },
            {
              "x": 19,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "2/10/2025",
              "label": "1",
              "pointId": "8768e111-59cf-4862-a78a-66c0e73470be"
            },
            {
              "x": 20,
              "y": 0,
              "z": "__NULL__",
              "pct": "__NULL__",
              "name": "2/17/2025",
              "dataLabels": {
                "enabled": false
              }
            },
            {
              "x": 21,
              "y": 0,
              "z": "__NULL__",
              "pct": "__NULL__",
              "name": "2/24/2025",
              "dataLabels": {
                "enabled": false
              }
            },
            {
              "x": 22,
              "y": 0,
              "z": "__NULL__",
              "pct": "__NULL__",
              "name": "3/10/2025",
              "dataLabels": {
                "enabled": false
              }
            },
            {
              "x": 23,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "3/24/2025",
              "label": "1",
              "pointId": "fe3edcd8-3d57-4721-b1cb-07fc15747d1d"
            },
            {
              "x": 24,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "7/7/2025",
              "label": "1",
              "pointId": "dfdd704d-7552-453f-abef-67539e8e6ed2"
            },
            {
              "x": 25,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "7/14/2025",
              "label": "1",
              "pointId": "31cc3ce7-5c9e-4e31-be87-dda8c7f925a9"
            },
            {
              "x": 26,
              "y": 1.0,
              "z": "2",
              "pct": "50%",
              "name": "8/4/2025",
              "label": "1",
              "pointId": "8aa657e7-0f67-427d-b053-c0223bc8c44e"
            },
            {
              "x": 27,
              "y": 0,
              "z": "__NULL__",
              "pct": "__NULL__",
              "name": "9/8/2025",
              "dataLabels": {
                "enabled": false
              }
            }
          ],
          "color": "rgba(199, 211, 227, 1)",
          "name": "Shared API creates",
          "index": 0,
          "legendIndex": 1
        }
      ],
      "filters": {
        "ee7a4e42-eb34-4e94-9acd-cd9257a8610c": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-03-18"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "a6883f2a-59db-40fb-a1a5-a061f0bbcef5": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-09-16"
              ]
            }
          ]
        },
        "60a5c917-26e0-4759-90ac-993a3798a99d": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-09-23"
              ]
            }
          ]
        },
        "9917c8dc-4e61-4c3c-8964-ebf4695b4b3d": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-10-14"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "e9cda0d2-81c6-47ff-ba67-922ab881538a": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-09-23"
              ]
            }
          ]
        },
        "e1ed3747-071b-4d65-9f44-d58a0f70b308": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-04-15"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "93de8cbf-c040-428e-99a2-e438e3c6ba25": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-02-10"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "dfdd704d-7552-453f-abef-67539e8e6ed2": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-07-07"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            }
          ]
        },
        "7f2e4b28-3823-490f-ba13-6e1099d0e936": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-06-17"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            }
          ]
        },
        "c01e8f55-b969-40b0-bff7-6ba9071464dd": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-02-17"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "31cc3ce7-5c9e-4e31-be87-dda8c7f925a9": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-07-14"
              ]
            }
          ]
        },
        "b2c08223-bace-4b6d-832f-37f2af5a7f09": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-02-05"
              ]
            }
          ]
        },
        "74245773-e21f-4ef6-9ba2-a2e0a1c4fcbe": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-06-10"
              ]
            }
          ]
        },
        "f352bd73-4a75-458a-8828-5717a4f6c8ba": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-03-04"
              ]
            }
          ]
        },
        "aa57293a-2791-48f4-b09f-7a59c6227139": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-07-08"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "93cd6baf-aa7e-4906-943f-abd368777472": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-11-18"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "f9058eec-e7df-4078-bd14-d5a44f167cd9": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-03-10"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "8768e111-59cf-4862-a78a-66c0e73470be": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-02-10"
              ]
            }
          ]
        },
        "97f4af66-f937-494a-aa7c-a2e13d3b7321": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-08-04"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "94789e6c-944c-4976-930e-3f14bb07d7f0": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-09-08"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "8aa657e7-0f67-427d-b053-c0223bc8c44e": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-08-04"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            }
          ]
        },
        "95190881-be17-4a54-8ad9-f2cb3286d48b": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-07-14"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "d80d5bc3-70e8-4bce-ab14-acbfcdacbfb5": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-04-15"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            }
          ]
        },
        "c32bbafa-94ca-4e19-bbc9-61cab532029c": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-04-22"
              ]
            }
          ]
        },
        "fe3edcd8-3d57-4721-b1cb-07fc15747d1d": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-03-24"
              ]
            }
          ]
        },
        "292b2f1a-f869-4d83-b8bd-4fb268072407": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-10-14"
              ]
            }
          ]
        },
        "e492591d-b8eb-4b60-bbab-4301c4f7a5d1": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-05-20"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "b78fe2dd-af31-46b2-8840-319d6849b7a0": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-01-08"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "f1ba80bb-f61d-4d04-b4d9-7e30b4c6e224": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-07-07"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "910f1d36-fbf6-48bf-be60-b3734aa43e50": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-11-18"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            }
          ]
        },
        "b3720b8b-ae75-4ced-8b03-0249f2f5356e": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-01-08"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            }
          ]
        },
        "0e8a0e72-ed0b-49a4-95a3-519e55f92d0c": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-03-11"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "4448f672-ea9f-4209-8a6f-43c925d05f54": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-02-12"
              ]
            }
          ]
        },
        "8b637272-5570-46cc-9293-c2968f52227d": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-06-10"
              ]
            }
          ]
        },
        "5d0c89ab-7889-4d71-848d-c96cc37c7663": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-08-05"
              ]
            }
          ]
        },
        "82e4c3f9-28d3-4960-8dfb-15a6d0dcca93": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-06-17"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "b782adf6-9113-43c7-826a-e105a33da576": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-01-01"
              ]
            }
          ]
        },
        "d35ccc70-aa52-4313-b9fd-8c907db179fa": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-03-11"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            }
          ]
        },
        "def87560-4782-44ee-a0b7-f91a771f7657": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-08-05"
              ]
            }
          ]
        },
        "a913a048-9ac1-4f2a-b30a-aa880e6b5c24": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-08-19"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "2fb6450a-2ea1-46f2-9839-c18645230788": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-02-24"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "71f6ac50-1da8-4db0-8018-edb4f6d40ddd": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-04-22"
              ]
            }
          ]
        },
        "9a32820b-0743-4dba-90f0-9f14385f795c": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-08-19"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            }
          ]
        },
        "d9b71218-395f-42cd-b5c8-6a0412625f5b": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2025-03-24"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "API creates"
              ]
            }
          ]
        },
        "78e75a21-275a-4433-b89b-35c3ba28fc75": {
          "operator": "AND",
          "filters": [
            {
              "alias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
              "operator": "EQ",
              "values": [
                "Shared API creates"
              ]
            },
            {
              "alias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
              "operator": "EQ",
              "values": [
                "2024-05-20"
              ]
            }
          ]
        }
      },
      "additionalFields": null
    },
    "queries": null,
    "compatibility": {
      "BUBBLE": false,
      "DONUT": false,
      "D3BUBBLE": true,
      "KPI": false,
      "GRID": true,
      "COLUMN": true,
      "STACKED_COLUMN": true,
      "SCATTER": false,
      "COLUMN_LINE": false,
      "PIE": false,
      "STACKED_BAR": true,
      "BAR": true,
      "FUNNEL": false,
      "AREA": true,
      "HEATMAP": true,
      "LINE": true,
      "GAUGE": false
    },
    "reportDefinition": {
      "gsReportMaster": {
        "createdDate": 1741885693794,
        "modifiedDate": 1742849750486,
        "deleted": false,
        "reportId": "83b5e7f7-87de-47f3-bb94-218ddf041dbb",
        "reportName": "API Creates WoW",
        "reportDescription": "",
        "sourceDetails": {
          "objectName": "product_usage_team_metrics_weekly__gc",
          "objectLabel": "Product Usage Team Metrics Weekly",
          "connectionId": "MDA",
          "connectionType": "MDA",
          "dataStoreType": "HAPOSTGRES"
        },
        "showFields": [
          {
            "fieldName": "Metric_Value__gc",
            "dbName": "metric_value__gc",
            "label": "Metric Value",
            "fieldAlias": "sum_of_product_usage_team_metrics_weekly__gc_Metric_Value__gc",
            "dataType": "number",
            "objectName": "product_usage_team_metrics_weekly__gc",
            "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
            "properties": {
              "required": false,
              "aggregatable": true,
              "pathLabel": "Product Usage Team Metrics Weekly [Metric Value]",
              "columnCalculationConfigs": [],
              "inlineEditable": false
            },
            "displayOrder": 4,
            "connectionId": "MDA",
            "connectionType": "MDA",
            "fieldType": "calculated",
            "displayName": "SUM of Metric Value",
            "rowGrouped": false,
            "pivoted": false,
            "scale": 0,
            "groupable": true,
            "sortable": true,
            "filterable": true,
            "expressionDetails": {
              "expressionType": "aggregationFunction",
              "expression": {
                "tokenType": "aggregationFunction",
                "key": "SUM",
                "outputDataType": "NUMBER"
              }
            }
          }
        ],
        "drillDownFields": [
          {
            "fieldName": "Name",
            "dbName": "gsd73342",
            "label": "Name",
            "fieldAlias": "relationship_Postman_Team_ID__gr_Name",
            "dataType": "string",
            "objectName": "relationship",
            "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
            "key": "Postman_Team_ID__gc_Name",
            "properties": {
              "required": true,
              "pathLabel": "Postman Team ID"
            },
            "fieldPath": {
              "lookupId": "Postman_Team_ID__gr",
              "legacyLookupId": "fcb50f21-f7f8-4290-b408-c23dc63093fc",
              "lookupName": "Postman_Team_ID__gr",
              "left": {
                "type": "BASE_FIELD",
                "fieldName": "Postman_Team_ID__gc",
                "dbName": "postman_team_id__gc",
                "label": "Postman Team ID",
                "objectName": "relationship",
                "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
                "hasLookup": false,
                "displayOrder": 0
              },
              "right": {
                "type": "BASE_FIELD",
                "fieldName": "Postman_Team_ID__gc",
                "dbName": "postman_team_id__gc",
                "label": "Postman Team ID",
                "objectName": "product_usage_team_metrics_weekly__gc",
                "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
                "hasLookup": false,
                "displayOrder": 0
              }
            },
            "connectionId": "MDA",
            "connectionType": "MDA",
            "fieldType": "field",
            "displayName": "Postman Team Name",
            "rowGrouped": false,
            "pivoted": false,
            "scale": 0,
            "groupable": true,
            "sortable": true,
            "filterable": true
          },
          {
            "fieldName": "Week_Timestamp__gc",
            "dbName": "week_timestamp__gc",
            "label": "Week Timestamp",
            "fieldAlias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
            "dataType": "date",
            "objectName": "product_usage_team_metrics_weekly__gc",
            "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
            "key": "Week_Timestamp__gc",
            "properties": {
              "required": false,
              "aggregatable": true,
              "pathLabel": "Product Usage Team Metrics Weekly [Week Timestamp]",
              "columnCalculationConfigs": []
            },
            "displayOrder": 1,
            "connectionId": "MDA",
            "connectionType": "MDA",
            "fieldType": "field",
            "displayName": "Week Timestamp",
            "rowGrouped": false,
            "pivoted": false,
            "orderByInfo": {
              "nulls": "LAST",
              "order": "ASC"
            },
            "scale": 0,
            "groupable": true,
            "sortable": true,
            "filterable": true
          },
          {
            "fieldName": "Metric_Name__gc",
            "dbName": "metric_name__gc",
            "label": "Metric Name",
            "fieldAlias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
            "dataType": "string",
            "objectName": "product_usage_team_metrics_weekly__gc",
            "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
            "key": "Metric_Name__gc",
            "properties": {
              "required": false,
              "aggregatable": true,
              "pathLabel": "Product Usage Team Metrics Weekly [Metric Name]",
              "columnCalculationConfigs": []
            },
            "displayOrder": 2,
            "connectionId": "MDA",
            "connectionType": "MDA",
            "fieldType": "field",
            "displayName": "Metric Name",
            "rowGrouped": false,
            "pivoted": false,
            "scale": 0,
            "groupable": true,
            "sortable": true,
            "filterable": true
          },
          "... [truncated]"
        ],
        "groupByFields": [
          {
            "fieldName": "Week_Timestamp__gc",
            "dbName": "week_timestamp__gc",
            "label": "Week Timestamp",
            "fieldAlias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
            "dataType": "date",
            "objectName": "product_usage_team_metrics_weekly__gc",
            "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
            "properties": {
              "required": false,
              "aggregatable": true,
              "pathLabel": "Product Usage Team Metrics Weekly [Week Timestamp]",
              "columnCalculationConfigs": []
            },
            "displayOrder": 1,
            "connectionId": "MDA",
            "connectionType": "MDA",
            "fieldType": "field",
            "displayName": "Week Timestamp",
            "rowGrouped": false,
            "pivoted": false,
            "scale": 0,
            "groupable": true,
            "sortable": true,
            "filterable": true
          },
          {
            "fieldName": "Metric_Name__gc",
            "dbName": "metric_name__gc",
            "label": "Metric Name",
            "fieldAlias": "product_usage_team_metrics_weekly__gc_Metric_Name__gc",
            "dataType": "string",
            "objectName": "product_usage_team_metrics_weekly__gc",
            "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
            "properties": {
              "required": false,
              "aggregatable": true,
              "pathLabel": "Product Usage Team Metrics Weekly [Metric Name]",
              "columnCalculationConfigs": [],
              "alwaysIncludeInColorMaster": true,
              "colorDisabled": false
            },
            "displayOrder": 2,
            "connectionId": "MDA",
            "connectionType": "MDA",
            "fieldType": "field",
            "displayName": "Metric Name",
            "rowGrouped": false,
            "pivoted": false,
            "scale": 0,
            "groupable": true,
            "sortable": true,
            "filterable": true,
            "expressionDetails": {}
          }
        ],
        "whereFilters": {
          "conditions": [
            {
              "locked": true,
              "filterAlias": "A",
              "comparisonOperator": "CONTAINS",
              "rightOperandType": "VALUE",
              "filterValue": {
                "value": [
                  "API creates"
                ]
              },
              "includeNulls": false,
              "includeEmptyValues": false,
              "anyAtGlobal": false,
              "global": false,
              "leftOperand": {
                "fieldName": "Metric_Name__gc",
                "dbName": "metric_name__gc",
                "label": "Metric Name",
                "dataType": "STRING",
                "objectName": "product_usage_team_metrics_weekly__gc",
                "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
                "objectLabel": "Product Usage Team Metrics Weekly",
                "key": "Metric_Name__gc",
                "properties": {
                  "sourceType": "STRING",
                  "pathLabel": "Metric Name"
                },
                "fieldType": "field",
                "rowGrouped": false,
                "pivoted": false,
                "groupable": false,
                "sortable": false,
                "filterable": false
              }
            }
          ],
          "expression": "A"
        },
        "havingFilters": {
          "conditions": [],
          "expression": ""
        },
        "orderByFields": [
          {
            "fieldName": "Week_Timestamp__gc",
            "dbName": "week_timestamp__gc",
            "label": "Week Timestamp",
            "fieldAlias": "product_usage_team_metrics_weekly__gc_Week_Timestamp__gc",
            "dataType": "date",
            "objectName": "product_usage_team_metrics_weekly__gc",
            "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
            "properties": {
              "required": false,
              "aggregatable": true,
              "pathLabel": "Product Usage Team Metrics Weekly [Week Timestamp]",
              "columnCalculationConfigs": []
            },
            "displayOrder": 1,
            "connectionId": "MDA",
            "connectionType": "MDA",
            "fieldType": "field",
            "displayName": "Week Timestamp",
            "rowGrouped": false,
            "pivoted": false,
            "orderByInfo": {
              "nulls": "LAST",
              "order": "ASC"
            },
            "scale": 0,
            "groupable": true,
            "sortable": true,
            "filterable": true
          }
        ],
        "pageSize": 50,
        "reportDisplayType": "LINE",
        "reportOptions": {
          "normalize": false,
          "cumulative": false,
          "enableMilestone": false,
          "enableStacking": false,
          "enableCumulativePercentage": false,
          "showLabels": true,
          "enableRGBColor": true,
          "enableDualYAxis": true,
          "enableTextWrap": false,
          "freezeFirstColumn": false,
          "enableRanking": false,
          "enableSmoothLine": false,
          "enableMissingDatapoints": false,
          "enableClientSideRowGrouping": false,
          "enableConditionalColoring": false,
          "enableAddRecord": false,
          "enableDataExport": true,
          "rowHeight": "S",
          "enableComments": false
        },
        "properties": {
          "creatable": true
        },
        "reportTypes": [],
        "reportType": "adhoc",
        "requestSource": "R360",
        "createdBy": "1P01Q1IO46XBY7CLHNINQ7K7RXGI6923IS2C",
        "modifiedBy": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
        "createdByName": "Andor Fuhrer",
        "modifiedByName": "Emma Johnson-Prabhakar",
        "createdDateStr": "2025-03-13 17:08:13 UTC",
        "modifiedDateStr": "2025-03-24 20:55:50 UTC"
      },
      "additionalGlobalFilters": {
        "conditions": [
          {
            "filterAlias": "G1",
            "logicalOperator": "AND",
            "comparisonOperator": "EQ",
            "rightOperandType": "VALUE",
            "filterValue": {
              "value": [
                "68639"
              ]
            },
            "anyAtGlobal": false,
            "global": false,
            "leftOperand": {
              "fieldName": "Postman_Team_ID__gc",
              "dbName": "postman_team_id__gc",
              "label": "Postman Team ID",
              "dataType": "LOOKUP",
              "objectName": "product_usage_team_metrics_weekly__gc",
              "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
              "objectLabel": "Product Usage Team Metrics Weekly",
              "fieldType": "Field",
              "rowGrouped": false,
              "pivoted": false,
              "groupable": false,
              "sortable": false,
              "filterable": false
            }
          }
        ],
        "expression": "G1"
      }
    },
    "age": {
      "timestamp": 1758014426032,
      "localizedLabel": "This data was last refreshed on 9/16/2025 2:20 AM (0 seconds ago)",
      "label": "This data was last refreshed on 9/16/2025 2:20 AM (0 seconds ago)."
    }
  },
  "duration": 485,
  "alerts": []
}
```


---

## Companies & Relationships

### GET /v1/api/describe/MDA/relationship
**URL:** `https://postman.us2.gainsightcloud.com/v1/api/describe/MDA/relationship?ci=MDA&ic=true&cl=2&ppos=true&op=META&ade=true`
**Query Parameters:** ci, ic, cl, ppos, op, ade
**Status Codes:** [200]
**Response Size:** 1223636 - 1223636 bytes

**Response:**
```json
{
  "result": true,
  "requestId": "854f039d-d6e9-4082-9b45-31fa5663904e",
  "data": {
    "relationship_type": {
      "objectId": "38f97cc2-1cb9-4082-bd7a-92dec18ad7f7",
      "objectName": "relationship_type",
      "dbName": "relationship_type_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "Relationship Type",
      "objectType": "Standard",
      "labelPlural": "Relationship Type",
      "keyPrefix": "1P06GH6NC0YFN2YXBD",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "EDITABLE",
      "schemaEditability": "EDITABLE",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": true,
      "fields": [
        {
          "fieldName": "Description",
          "dbName": "gsd57823",
          "label": "Description",
          "dataType": "RICHTEXTAREA",
          "objectName": "relationship_type",
          "objectDBName": "relationship_type_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Relationship Type",
          "meta": {
            "properties": {
              "sourceType": "RICHTEXTAREA",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": false,
            "sortable": false,
            "groupable": false,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": true,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 2,
            "length": 32768,
            "originalDataType": "richtextarea",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": false,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "CreatedDate",
          "dbName": "gsd29121",
          "label": "Created Date",
          "dataType": "DATETIME",
          "objectName": "relationship_type",
          "objectDBName": "relationship_type_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Relationship Type",
          "meta": {
            "properties": {
              "sourceType": "DATETIME",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "originalDataType": "datetime",
            "valueType": "DATETIME",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "description": "Created Date",
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "GsIngestionSource",
          "dbName": "gsd59765",
          "label": "GS Ingestion Source",
          "dataType": "STRING",
          "objectName": "relationship_type",
          "objectDBName": "relationship_type_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Relationship Type",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "Relationship Type Standard Object",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": false,
      "frequentlyUsed": false,
      "order": 0,
      "componentCode": "360",
      "external": false
    },
    "scorecard_master": {
      "objectId": "7bd118ed-218c-44f5-aa86-43c897fa005d",
      "objectName": "scorecard_master",
      "dbName": "sc_master_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "Scorecard Master",
      "objectType": "System",
      "labelPlural": "Scorecard Master",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "READONLY",
      "schemaEditability": "READONLY",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": false,
      "fields": [
        {
          "fieldName": "Gsid",
          "dbName": "gsid",
          "label": "GSID",
          "dataType": "GSID",
          "objectName": "scorecard_master",
          "objectDBName": "sc_master_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Scorecard Master",
          "meta": {
            "properties": {
              "sourceType": "GSID",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "gsid",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "Name",
          "dbName": "name",
          "label": "Name",
          "dataType": "STRING",
          "objectName": "scorecard_master",
          "objectDBName": "sc_master_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Scorecard Master",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": true,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": true,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "Description",
          "dbName": "description",
          "label": "Description",
          "dataType": "STRING",
          "objectName": "scorecard_master",
          "objectDBName": "sc_master_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Scorecard Master",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": false,
      "frequentlyUsed": false,
      "order": 0,
      "componentCode": "hs",
      "external": false
    },
    "company_person": {
      "objectId": "d4adc849-3da8-4b7b-bf9c-259ee93f1e24",
      "objectName": "company_person",
      "dbName": "company_person_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "Company Person",
      "objectType": "Standard",
      "labelPlural": "Company Person",
      "keyPrefix": "1C01QVIMIKMMFNKZ27",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "EDITABLE",
      "schemaEditability": "EDITABLE",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": true,
      "fields": [
        {
          "fieldName": "Mailing_City__gc",
          "dbName": "mailing_city__gc",
          "label": "Mailing City",
          "dataType": "STRING",
          "objectName": "company_person",
          "objectDBName": "company_person_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Company Person",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "CUSTOM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 500,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "LastNpsScore",
          "dbName": "lastnpsscore",
          "label": "Last NPS Score",
          "dataType": "NUMBER",
          "objectName": "company_person",
          "objectDBName": "company_person_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Company Person",
          "meta": {
            "properties": {
              "sourceType": "NUMBER",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "originalDataType": "number",
            "valueType": "NUMBER",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "description": "Update Last NPS Score with current value from  nps_survey_response NPSScore",
            "aggregatable": true,
            "colAttributeType": 1,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "EmailCompany",
          "dbName": "gsd22804",
          "label": "Email (Company)",
          "dataType": "EMAIL",
          "objectName": "company_person",
          "objectDBName": "company_person_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Company Person",
          "meta": {
            "properties": {
              "sourceType": "EMAIL",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "email",
            "valueType": "STRING",
            "mappings": {
              "GAINSIGHT": {
                "key": "GS_COMPANY_PERSON_EMAIL",
                "dataType": "email"
              }
            },
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": true,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "Company Person Standard Object",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": false,
      "frequentlyUsed": false,
      "order": 0,
      "componentCode": "PM",
      "external": false
    },
    "gsrecordtype": {
      "objectId": "d372e172-05ba-4ebf-a965-071c1bcd999d",
      "objectName": "gsrecordtype",
      "dbName": "gs_record_type_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "GS Record Type",
      "objectType": "Standard",
      "labelPlural": "GS Record Type",
      "keyPrefix": "1RX3GXAMVR7Y2O0L3K",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "EDITABLE",
      "schemaEditability": "READONLY",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": false,
      "fields": [
        {
          "fieldName": "Gsid",
          "dbName": "gsid",
          "label": "GSID",
          "dataType": "GSID",
          "objectName": "gsrecordtype",
          "objectDBName": "gs_record_type_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "GS Record Type",
          "meta": {
            "properties": {
              "sourceType": "GSID",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "gsid",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "ExternalId",
          "dbName": "external_id",
          "label": "External Id",
          "dataType": "STRING",
          "objectName": "gsrecordtype",
          "objectDBName": "gs_record_type_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "GS Record Type",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "Name",
          "dbName": "name",
          "label": "Name",
          "dataType": "STRING",
          "objectName": "gsrecordtype",
          "objectDBName": "gs_record_type_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "GS Record Type",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": true,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": true,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "Gainsight Standard Record Type Object",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": false,
      "frequentlyUsed": false,
      "order": 0,
      "componentCode": "RC",
      "external": false
    },
    "person": {
      "objectId": "c5939a5d-b53d-4903-a767-56526e9aae6b",
      "objectName": "person",
      "dbName": "person_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "Person",
      "objectType": "Standard",
      "labelPlural": "Person",
      "keyPrefix": "1P04XVWNX00AA6V0E1",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "EDITABLE",
      "schemaEditability": "EDITABLE",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": true,
      "fields": [
        {
          "fieldName": "SFDC_Title__gc",
          "dbName": "sfdc_title__gc",
          "label": "SFDC Title",
          "dataType": "STRING",
          "objectName": "person",
          "objectDBName": "person_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Person",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "CUSTOM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 500,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "DynamicResolutionKey",
          "dbName": "resolution_key__gs",
          "label": "Dynamic Resolution Key",
          "dataType": "STRING",
          "objectName": "person",
          "objectDBName": "person_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Person",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "description": "Dynamic Resolution Key",
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "GsIngestionSource",
          "dbName": "gsd21319",
          "label": "GS Ingestion Source",
          "dataType": "STRING",
          "objectName": "person",
          "objectDBName": "person_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Person",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "Person Standard Object",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": false,
      "frequentlyUsed": false,
      "order": 0,
      "componentCode": "PM",
      "external": false
    },
    "gs_pricebook": {
      "objectId": "17586f5f-1f57-4946-96e3-2bf2684f76b0",
      "objectName": "gs_pricebook",
      "dbName": "gs_pricebook_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "GS Pricebook",
      "objectType": "Standard",
      "labelPlural": "GS Pricebook",
      "keyPrefix": "1RX5KAJZPAPEGNZS2T",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "EDITABLE",
      "schemaEditability": "EDITABLE",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": true,
      "fields": [
        {
          "fieldName": "Gsid",
          "dbName": "gsid",
          "label": "GSID",
          "dataType": "GSID",
          "objectName": "gs_pricebook",
          "objectDBName": "gs_pricebook_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "GS Pricebook",
          "meta": {
            "properties": {
              "sourceType": "GSID",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "gsid",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "ExternalId",
          "dbName": "external_id",
          "label": "External Id",
          "dataType": "STRING",
          "objectName": "gs_pricebook",
          "objectDBName": "gs_pricebook_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "GS Pricebook",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "Name",
          "dbName": "name",
          "label": "Name",
          "dataType": "STRING",
          "objectName": "gs_pricebook",
          "objectDBName": "gs_pricebook_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "GS Pricebook",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": true,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": true,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "Gainsight Standard PriceBook Object",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": false,
      "frequentlyUsed": false,
      "order": 0,
      "componentCode": "RC",
      "external": false
    },
    "scoring_scheme_definition": {
      "objectId": "a7e063e9-d68e-452d-84f8-1e9af713e79e",
      "objectName": "scoring_scheme_definition",
      "dbName": "sc_scheme_definition_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "Scoring Scheme Definition",
      "objectType": "System",
      "labelPlural": "Scoring Scheme Definition",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "READONLY",
      "schemaEditability": "READONLY",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": false,
      "fields": [
        {
          "fieldName": "Gsid",
          "dbName": "gsid",
          "label": "GSID",
          "dataType": "GSID",
          "objectName": "scoring_scheme_definition",
          "objectDBName": "sc_scheme_definition_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Scoring Scheme Definition",
          "meta": {
            "properties": {
              "sourceType": "GSID",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "gsid",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "Name",
          "dbName": "name",
          "label": "Name",
          "dataType": "STRING",
          "objectName": "scoring_scheme_definition",
          "objectDBName": "sc_scheme_definition_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Scoring Scheme Definition",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": true,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": true,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "Label",
          "dbName": "label",
          "label": "Label",
          "dataType": "STRING",
          "objectName": "scoring_scheme_definition",
          "objectDBName": "sc_scheme_definition_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Scoring Scheme Definition",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": false,
      "frequentlyUsed": false,
      "order": 0,
      "componentCode": "hs",
      "external": false
    },
    "gsuser": {
      "objectId": "d1329473-f5de-4e5a-8284-ed981334d36c",
      "objectName": "gsuser",
      "dbName": "user_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "User",
      "objectType": "Standard",
      "labelPlural": "User",
      "keyPrefix": "1P01Q1IO46XBY7CLHN",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "EDITABLE",
      "schemaEditability": "EDITABLE",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": true,
      "fields": [
        {
          "fieldName": "Email",
          "dbName": "gsd17277",
          "label": "Email",
          "dataType": "EMAIL",
          "objectName": "gsuser",
          "objectDBName": "user_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "User",
          "meta": {
            "properties": {
              "sourceType": "EMAIL",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "email",
            "valueType": "STRING",
            "mappings": {
              "GAINSIGHT": {
                "key": "GS_USER_EMAIL",
                "dataType": "email"
              }
            },
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": true,
            "tracked": true,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "LastName",
          "dbName": "gsd3337",
          "label": "Last Name",
          "dataType": "STRING",
          "objectName": "gsuser",
          "objectDBName": "user_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "User",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": true,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "Manager",
          "dbName": "gsd91561",
          "label": "Manager",
          "dataType": "LOOKUP",
          "objectName": "gsuser",
          "objectDBName": "user_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "User",
          "meta": {
            "properties": {
              "sourceType": "GSID",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              },
              "SEARCH_CONTROLLER": "AUTO_SUGGEST"
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": true,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": true,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "lookup",
            "valueType": "STRING",
            "mappings": {
              "GAINSIGHT": {
                "key": "GS_USER_ID",
                "dataType": "gsid"
              }
            },
            "lookupDetail": {
              "lookupName": "Manager__gr",
              "fieldName": "Gsid",
              "fieldDBName": "gsid",
              "fieldLabel": "GSID",
              "lookupId": "Manager__gr",
              "lookupObjects": [
                {
                  "id": "d1329473-f5de-4e5a-8284-ed981334d36c",
                  "objectName": "gsuser",
                  "dbName": "user_3a5744bf41214e2eab6fee7bf36fc524",
                  "label": "User",
                  "namedFieldDetails": {
                    "name": "gsd14784",
                    "hidden": false,
                    "indexed": true,
                    "unique": false,
                    "primary": false,
                    "distinctMemberCount": 0,
                    "dimensionBrowserCollection": "SELF",
                    "alignment": "LEFT",
                    "aggFunction": "SUM",
                    "numberType": "NUMBER",
                    "decimalPlaces": 0,
                    "thousandSeparatorUsed": false,
                    "negativeNumber": "MINUSVALUE",
                    "systemDefined": false,
                    "encrypted": false,
                    "deleted": false,
                    "mappings": {
                      "GAINSIGHT": {
                        "key": "GS_USER_NAME",
                        "dataType": "string"
                      }
                    },
                    "hasLookup": false,
                    "gdmFormulaColumn": false,
                    "gdmFreeFormEditor": false,
                    "required": false,
                    "editedColumn": false,
                    "newColumn": false,
                    "fieldName": "Name",
                    "withTimeZone": false,
                    "DBName": "gsd14784",
                    "datatype": "string",
                    "colattribtype": 0,
                    "DisplayName": "Name",
                    "FieldSchemaEditabilityType": "Fixed",
                    "FieldDataEditabilityType": "All",
                    "FieldGroupType": "STANDARD",
                    "isSelfLookupField": false,
                    "resolutionKeys": [],
                    "isSearchable": true,
                    "isReferenceField": false,
                    "isTracked": true,
                    "isNamedField": true,
                    "isExternalId": false,
                    "isPermissionAttribute": false,
                    "isCompressed": false,
                    "isAdvancedFormula": false
                  }
                }
              ]
            },
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "User Standard Object",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": true,
      "frequentlyUsed": false,
      "order": 0,
      "auditCollectionName": "gsuser_audit_3a5744bf41214e2eab6fee7bf36fc524",
      "componentCode": "UM",
      "external": false
    },
    "company": {
      "objectId": "eb5c67cf-5fe7-4255-a0b7-c6345dbddf82",
      "objectName": "company",
      "dbName": "company_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "Company",
      "objectType": "Standard",
      "labelPlural": "Company",
      "keyPrefix": "1P02V9QEJ80HV8XSMY",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "EDITABLE",
      "schemaEditability": "EDITABLE",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": true,
      "fields": [
        {
          "fieldName": "OriginalContractDate",
          "dbName": "gsd64242",
          "label": "Original Contract Date",
          "dataType": "DATE",
          "objectName": "company",
          "objectDBName": "company_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Company",
          "meta": {
            "properties": {
              "sourceType": "DATE",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "originalDataType": "date",
            "valueType": "DATE",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "Google_Drive_Account_Folder__gc",
          "dbName": "gsd73832",
          "label": "Google Drive: Account Folder",
          "dataType": "URL",
          "objectName": "company",
          "objectDBName": "company_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Company",
          "meta": {
            "properties": {
              "sourceType": "URL",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "CUSTOM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 500,
            "originalDataType": "url",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "format": {
              "dateFormat": "M/D/YYYY"
            },
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "Csm",
          "dbName": "gsd9655",
          "label": "CSM",
          "dataType": "LOOKUP",
          "objectName": "company",
          "objectDBName": "company_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Company",
          "meta": {
            "properties": {
              "sourceType": "GSID",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              },
              "SEARCH_CONTROLLER": "AUTO_SUGGEST"
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": true,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "lookup",
            "valueType": "STRING",
            "mappings": {
              "GAINSIGHT": {
                "key": "GS_USER_ID",
                "dataType": "gsid"
              }
            },
            "lookupDetail": {
              "lookupName": "Csm__gr",
              "fieldName": "Gsid",
              "fieldDBName": "gsid",
              "fieldLabel": "GSID",
              "lookupId": "c508d49d-d4bc-4b48-9c5b-f212e61196ce",
              "lookupObjects": [
                {
                  "id": "d1329473-f5de-4e5a-8284-ed981334d36c",
                  "objectName": "gsuser",
                  "dbName": "user_3a5744bf41214e2eab6fee7bf36fc524",
                  "label": "User",
                  "namedFieldDetails": {
                    "name": "gsd14784",
                    "hidden": false,
                    "indexed": true,
                    "unique": false,
                    "primary": false,
                    "distinctMemberCount": 0,
                    "dimensionBrowserCollection": "SELF",
                    "alignment": "LEFT",
                    "aggFunction": "SUM",
                    "numberType": "NUMBER",
                    "decimalPlaces": 0,
                    "thousandSeparatorUsed": false,
                    "negativeNumber": "MINUSVALUE",
                    "systemDefined": false,
                    "encrypted": false,
                    "deleted": false,
                    "mappings": {
                      "GAINSIGHT": {
                        "key": "GS_USER_NAME",
                        "dataType": "string"
                      }
                    },
                    "hasLookup": false,
                    "gdmFormulaColumn": false,
                    "gdmFreeFormEditor": false,
                    "required": false,
                    "editedColumn": false,
                    "newColumn": false,
                    "fieldName": "Name",
                    "withTimeZone": false,
                    "DBName": "gsd14784",
                    "datatype": "string",
                    "colattribtype": 0,
                    "DisplayName": "Name",
                    "FieldSchemaEditabilityType": "Fixed",
                    "FieldDataEditabilityType": "All",
                    "FieldGroupType": "STANDARD",
                    "isSelfLookupField": false,
                    "resolutionKeys": [],
                    "isSearchable": true,
                    "isReferenceField": false,
                    "isTracked": true,
                    "isNamedField": true,
                    "isExternalId": false,
                    "isPermissionAttribute": false,
                    "isCompressed": false,
                    "isAdvancedFormula": false
                  }
                }
              ]
            },
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "onDeleteOperation": "None",
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": true,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "Company Standard Object",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": false,
      "frequentlyUsed": false,
      "order": 0,
      "auditCollectionName": "company_audit_3a5744bf41214e2eab6fee7bf36fc524",
      "componentCode": "360",
      "external": false
    },
    "relationship": {
      "objectId": "b12ad427-06e1-4e0e-b878-381fb0b04207",
      "objectName": "relationship",
      "dbName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "Relationship",
      "objectType": "Standard",
      "labelPlural": "Relationship",
      "keyPrefix": "1P05VZVGZ526G7QVNW",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "EDITABLE",
      "schemaEditability": "EDITABLE",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": true,
      "fields": [
        {
          "fieldName": "ModifiedDate",
          "dbName": "gsd21337",
          "label": "Modified Date",
          "dataType": "DATETIME",
          "objectName": "relationship",
          "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Relationship",
          "meta": {
            "properties": {
              "sourceType": "DATETIME",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "originalDataType": "datetime",
            "valueType": "DATETIME",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "description": "Modified Date",
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "Mrr",
          "dbName": "gsm37642",
          "label": "MRR",
          "dataType": "CURRENCY",
          "objectName": "relationship",
          "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Relationship",
          "meta": {
            "properties": {
              "sourceType": "CURRENCY",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 2,
            "originalDataType": "currency",
            "valueType": "DOUBLE",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 1,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "CreatedDate",
          "dbName": "gsd78826",
          "label": "Created Date",
          "dataType": "DATETIME",
          "objectName": "relationship",
          "objectDBName": "relationship_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Relationship",
          "meta": {
            "properties": {
              "sourceType": "DATETIME",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": true,
            "decimalPlaces": 0,
            "originalDataType": "datetime",
            "valueType": "DATETIME",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "description": "Created Date",
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "Relationship Standard Object",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": false,
      "frequentlyUsed": false,
      "order": 0,
      "auditCollectionName": "relationship_audit_3a5744bf41214e2eab6fee7bf36fc524",
      "componentCode": "360",
      "external": false
    },
    "relationship_person": {
      "objectId": "15518566-bf33-47c7-9579-a6215f2d87f7",
      "objectName": "relationship_person",
      "dbName": "relationship_person_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "Relationship Person",
      "objectType": "Standard",
      "labelPlural": "Relationship Person",
      "keyPrefix": "1C031XSWLHXA9HD4EZ",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "EDITABLE",
      "schemaEditability": "EDITABLE",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": true,
      "fields": [
        {
          "fieldName": "User_Role__gc",
          "dbName": "user_role__gc",
          "label": "Redshift User Role",
          "dataType": "STRING",
          "objectName": "relationship_person",
          "objectDBName": "relationship_person_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Relationship Person",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "CUSTOM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 500,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "LastNpsScore",
          "dbName": "lastnpsscore",
          "label": "Last NPS Score",
          "dataType": "NUMBER",
          "objectName": "relationship_person",
          "objectDBName": "relationship_person_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Relationship Person",
          "meta": {
            "properties": {
              "sourceType": "NUMBER",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "originalDataType": "number",
            "valueType": "NUMBER",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "description": "Update Last NPS Score with current value from  nps_survey_response NPSScore",
            "aggregatable": true,
            "colAttributeType": 1,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "IsPrimaryCompany",
          "dbName": "gsd16088",
          "label": "Primary Company",
          "dataType": "BOOLEAN",
          "objectName": "relationship_person",
          "objectDBName": "relationship_person_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Relationship Person",
          "meta": {
            "properties": {
              "sourceType": "BOOLEAN",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "originalDataType": "boolean",
            "valueType": "BOOLEAN",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "Relationship Person Standard Object",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": false,
      "frequentlyUsed": false,
      "order": 0,
      "componentCode": "PM",
      "external": false
    },
    "gs_opportunity": {
      "objectId": "6be2c84c-c25f-4ce8-9cdf-2afcd7c1408e",
      "objectName": "gs_opportunity",
      "dbName": "gs_opportunity_3a5744bf41214e2eab6fee7bf36fc524",
      "label": "GS Opportunity",
      "objectType": "Standard",
      "labelPlural": "GS Opportunity",
      "keyPrefix": "1RX1DEXR4Q70CSV00B",
      "source": "MDA",
      "dataStore": "HAPOSTGRES",
      "cdcEnabled": true,
      "transactional": false,
      "dataEditability": "EDITABLE",
      "schemaEditability": "EDITABLE",
      "hidden": false,
      "deleted": false,
      "multiCurrencySupported": true,
      "fields": [
        {
          "fieldName": "Gsid",
          "dbName": "gsid",
          "label": "GSID",
          "dataType": "GSID",
          "objectName": "gs_opportunity",
          "objectDBName": "gs_opportunity_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "GS Opportunity",
          "meta": {
            "properties": {
              "sourceType": "GSID",
              "actualDataEditability": {
                "accessible": true,
                "createable": false,
                "updateable": false
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": false,
            "updateable": false,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": true,
            "dependentPicklist": false,
            "fieldGroupType": "SYSTEM",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "gsid",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "ExternalId",
          "dbName": "external_id",
          "label": "External Id",
          "dataType": "STRING",
          "objectName": "gs_opportunity",
          "objectDBName": "gs_opportunity_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "GS Opportunity",
          "meta": {
            "properties": {
              "sourceType": "STRING",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              }
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": false,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "string",
            "valueType": "STRING",
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        {
          "fieldName": "Company",
          "dbName": "company",
          "label": "Company",
          "dataType": "LOOKUP",
          "objectName": "gs_opportunity",
          "objectDBName": "gs_opportunity_3a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "GS Opportunity",
          "meta": {
            "properties": {
              "sourceType": "GSID",
              "actualDataEditability": {
                "accessible": true,
                "createable": true,
                "updateable": true
              },
              "SEARCH_CONTROLLER": "AUTO_SUGGEST"
            },
            "accessible": true,
            "filterable": true,
            "sortable": true,
            "groupable": true,
            "createable": true,
            "updateable": true,
            "externalId": false,
            "formulaField": false,
            "hasLookup": true,
            "required": false,
            "richText": false,
            "nillable": true,
            "readOnly": false,
            "dependentPicklist": false,
            "fieldGroupType": "STANDARD",
            "selfLookup": false,
            "gdmFormulaColumn": false,
            "gdmFreeFormEditor": false,
            "indexed": false,
            "decimalPlaces": 0,
            "length": 255,
            "originalDataType": "lookup",
            "valueType": "STRING",
            "mappings": {
              "GAINSIGHT": {
                "key": "GS_COMPANY_ID",
                "dataType": "gsid"
              }
            },
            "lookupDetail": {
              "lookupName": "Company__gr",
              "fieldName": "Gsid",
              "fieldDBName": "gsid",
              "fieldLabel": "GSID",
              "lookupId": "ef2e5f57-9953-423a-a392-c6b7b2252593",
              "lookupObjects": [
                {
                  "id": "eb5c67cf-5fe7-4255-a0b7-c6345dbddf82",
                  "objectName": "company",
                  "dbName": "company_3a5744bf41214e2eab6fee7bf36fc524",
                  "label": "Company",
                  "namedFieldDetails": {
                    "name": "gsd73733",
                    "hidden": false,
                    "indexed": false,
                    "unique": false,
                    "primary": false,
                    "distinctMemberCount": 0,
                    "dimensionBrowserCollection": "SELF",
                    "alignment": "LEFT",
                    "aggFunction": "SUM",
                    "numberType": "NUMBER",
                    "decimalPlaces": 0,
                    "thousandSeparatorUsed": false,
                    "negativeNumber": "MINUSVALUE",
                    "systemDefined": false,
                    "encrypted": false,
                    "deleted": false,
                    "mappings": {
                      "GAINSIGHT": {
                        "key": "GS_COMPANY_NAME",
                        "dataType": "string"
                      }
                    },
                    "hasLookup": false,
                    "gdmFormulaColumn": false,
                    "gdmFreeFormEditor": false,
                    "required": true,
                    "editedColumn": false,
                    "newColumn": false,
                    "fieldName": "Name",
                    "withTimeZone": false,
                    "DBName": "gsd73733",
                    "datatype": "string",
                    "colattribtype": 0,
                    "DisplayName": "Name",
                    "FieldSchemaEditabilityType": "Fixed",
                    "FieldDataEditabilityType": "All",
                    "FieldGroupType": "STANDARD",
                    "isSelfLookupField": false,
                    "resolutionKeys": [],
                    "isSearchable": true,
                    "isReferenceField": false,
                    "isTracked": false,
                    "isNamedField": true,
                    "isExternalId": false,
                    "isPermissionAttribute": false,
                    "isCompressed": false,
                    "isAdvancedFormula": false
                  }
                }
              ]
            },
            "hidden": false,
            "deleted": false,
            "nameField": false,
            "aggregatable": true,
            "onDeleteOperation": "None",
            "colAttributeType": 0,
            "withTimeZone": false,
            "resolutionKeys": [],
            "searchable": false,
            "tracked": false,
            "eventable": false,
            "advancedFormula": false
          }
        },
        "... [truncated]"
      ],
      "description": "Gainsight Standard Opportunity Object",
      "accessible": false,
      "createable": false,
      "updateable": false,
      "eventEnabled": false,
      "frequentlyUsed": false,
      "order": 0,
      "componentCode": "RC",
      "external": false
    }
  }
}
```


### GET /v1/ui/Relationship360
**URL:** `https://postman.us2.gainsightcloud.com/v1/ui/Relationship360?rid=1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU`
**Query Parameters:** rid
**Status Codes:** [200]
**Response Size:** 547206 - 547206 bytes

**Response:**
```json
<!DOCTYPE html>
<html lang="en" class="gs-ui-nxt">
<head>
    <meta charset="UTF-8"/>
    <meta http-equiv="x-ua-compatible" content="ie=edge"/>
    <meta name="description" content=""/>
    <meta name="viewport" content="width=device-width, initial-scale=1"/>
    <link rel="shortcut icon" type="image/x-icon" href="https://gainsight-public.s3.amazonaws.com/native/v1/img/gainsight-favicon.ico">
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https:... [truncated]
```


### GET /v2/galaxy/bootstrap/consumption/config/R360
**URL:** `https://postman.us2.gainsightcloud.com/v2/galaxy/bootstrap/consumption/config/R360`
**Status Codes:** [200]
**Response Size:** 1443 - 1443 bytes

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "a6701706-2193-47c5-95b2-36e279a28e20",
  "data": {
    "feature-toggle": [
      {
        "exists": true,
        "featureId": "disable_report_export_for_dashbo_94184fbdeb38426884784509e6ed084e",
        "featureName": "DISABLE_REPORT_EXPORT_FOR_DASHBOARDS_R360",
        "variationId": "true_variation_c476cff5a17d4569ae184de5aa181acd",
        "variationName": "True Variation",
        "variationType": "BOOLEAN",
        "value": true
      },
      {
        "exists": true,
        "featureId": "messenger_0e8807b95d9b486bb1f557760c6aa024",
        "featureName": "messenger",
        "variationId": "true_variation_f69d6954c6d446168dc8c78279bb607c",
        "variationName": "True Variation",
        "variationType": "BOOLEAN",
        "value": true
      },
      {
        "exists": true,
        "featureId": "sponsor_tracking_5c3e42e983b243fda51ad8d78b9d7bac",
        "featureName": "SPONSOR_TRACKING",
        "variationId": "false_variation_2ab5f252ed284f2eb23b18b13788dd18",
        "variationName": "False Variation",
        "variationType": "BOOLEAN",
        "value": false
      },
      "... [truncated]"
    ]
  },
  "message": null,
  "localizedMessage": null
}
```


### POST /v2/galaxy/assignment/resolve/cid
**URL:** `https://postman.us2.gainsightcloud.com/v2/galaxy/assignment/resolve/cid`
**Status Codes:** [200]
**Response Size:** 58211 - 58211 bytes

**Request Body:**
```json
{
  "relationshipId": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU",
  "entityId": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU",
  "entityType": "relationship",
  "sharingType": "internal"
}
```

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "cdb97fbd-407d-44b9-a00f-0a571281e698",
  "data": {
    "layoutResolverDTO": {
      "accountId": "0011K00002H9qNmQAJ",
      "companyId": "1P02V9QEJ80HV8XSMYQMNQH6Y22550U5H6TR",
      "relationshipId": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU",
      "relationshipTypeId": "1P06GH6NC0YFN2YXBDUFZX2FIJADKKBZ3AD3",
      "entityType": "Relationship",
      "sharingType": "Internal",
      "userId": "1P01Q1IO46XBY7CLHN2UBT890O1790B43NVA",
      "entityId": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU",
      "isSfdcResolutionRequired": false,
      "skipSpacesInvitationCheck": false,
      "mini360": false
    },
    "layout": {
      "name": "V2 Postman Team Layout",
      "layoutId": "84441ebc-079b-4e3d-85f0-d0010b5a3eed",
      "description": "Updates to the CSM Summary ",
      "sections": [
        {
          "sectionId": "5ea4671a-3727-46ee-a1dc-797989a65f97",
          "label": "CSM Summary",
          "config": {
            "widgets": [
              {
                "label": "CSM",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w17",
                "description": null,
                "config": {
                  "fieldName": "Csm",
                  "dataType": "LOOKUP",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": true,
                    "required": null,
                    "navigationConfig": null,
                    "width": 4,
                    "rollup": null,
                    "requiredDisabled": false,
                    "SEARCH_CONTROLLER": {
                      "objectName": "gsuser",
                      "fields": [
                        "Name"
                      ]
                    }
                  },
                  "formatOptions": {
                    "type": null,
                    "numericalSummarization": null
                  },
                  "scale": 0,
                  "fieldPath": null,
                  "key": "Csm",
                  "lookupDisplayField": {
                    "fieldName": "Name",
                    "label": "Name",
                    "dataType": "STRING",
                    "objectName": "gsuser",
                    "objectLabel": "User",
                    "forcedDataType": null,
                    "objectId": "d1329473-f5de-4e5a-8284-ed981334d36c",
                    "type": "BASE_FIELD",
                    "key": "Csm_Name",
                    "fieldPath": {
                      "lookupId": "Csm__gr",
                      "lookupName": "Csm__gr",
                      "legacyLookupId": "f3541dcc-61bf-4046-8352-9c51218d3206",
                      "left": {
                        "type": "BASE_FIELD",
                        "fieldName": "Gsid",
                        "label": "GSID",
                        "objectName": "gsuser",
                        "hasLookup": false,
                        "displayOrder": 0
                      },
                      "right": {
                        "type": "BASE_FIELD",
                        "fieldName": "Csm",
                        "label": "CSM",
                        "objectName": "relationship",
                        "hasLookup": false,
                        "displayOrder": 0
                      },
                      "fieldPath": null
                    }
                  }
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 8,
                  "y": 0
                },
                "displayOrder": 0
              },
              {
                "label": "Logo",
                "widgetCategory": "Standard",
                "widgetType": "CR",
                "subType": "IMAGE",
                "config": {
                  "fieldName": "Logo",
                  "objectName": "relationship",
                  "dataType": "IMAGE",
                  "properties": {
                    "editable": true
                  }
                },
                "itemId": "w30",
                "className": null,
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 4,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 0,
                  "y": 0
                },
                "displayOrder": 1
              },
              {
                "label": "Postman Team ARR",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w16",
                "description": null,
                "config": {
                  "fieldName": "Postman_Team_ARR__gc",
                  "dataType": "CURRENCY",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null,
                    "width": 4,
                    "rollup": null,
                    "requiredDisabled": false,
                    "SEARCH_CONTROLLER": {
                      "fields": []
                    }
                  },
                  "formatOptions": {
                    "type": "CURRENCY",
                    "numericalSummarization": "None"
                  },
                  "scale": 0,
                  "fieldPath": null,
                  "key": "Postman_Team_ARR__gc",
                  "lookupDisplayField": null
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 12,
                  "y": 0
                },
                "displayOrder": 2
              },
              {
                "label": "AE",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w65",
                "description": null,
                "config": {
                  "fieldName": "Name",
                  "dataType": "STRING",
                  "objectName": "gsuser",
                  "objectLabel": "User",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null,
                    "width": 4,
                    "rollup": null,
                    "requiredDisabled": false,
                    "SEARCH_CONTROLLER": {
                      "fields": []
                    }
                  },
                  "formatOptions": {
                    "type": null,
                    "numericalSummarization": null
                  },
                  "scale": 0,
                  "fieldPath": {
                    "lookupId": "Account_Executive__gr",
                    "lookupName": "Account_Executive__gr",
                    "legacyLookupId": "4a9a5d70-e461-4d85-8a8a-f94898e7230e",
                    "left": {
                      "type": "BASE_FIELD",
                      "fieldName": "Gsid",
                      "label": "GSID",
                      "objectName": "gsuser",
                      "hasLookup": false,
                      "displayOrder": 0
                    },
                    "right": {
                      "type": "BASE_FIELD",
                      "fieldName": "Account_Executive__gc",
                      "label": "Account Executive",
                      "objectName": "relationship",
                      "hasLookup": false,
                      "displayOrder": 0
                    },
                    "fieldPath": null
                  },
                  "key": "Account_Executive__gc_Name",
                  "lookupDisplayField": null
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 4,
                  "y": 0
                },
                "displayOrder": 3
              },
              {
                "label": "Health Score with History",
                "widgetCategory": "Standard",
                "widgetType": "HEALTH_SCORE",
                "subType": "HEALTH_SCORE_METRIC_AND_HISTORY",
                "config": {
                  "isViewMeasure": false
                },
                "itemId": "w10",
                "className": "",
                "dimensionDetails": {
                  "rows": 6,
                  "cols": 8,
                  "maxItemCols": 12,
                  "maxItemRows": 6,
                  "minItemCols": 8,
                  "minItemRows": 6
                },
                "mini360Dimensions": {
                  "rows": 6,
                  "cols": 24
                },
                "axisDetails": {
                  "x": 16,
                  "y": 0
                },
                "displayOrder": 4
              },
              {
                "label": "Renewal Date",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w24",
                "config": {
                  "fieldName": "Renewal_Date__gc",
                  "dataType": "DATE",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": null,
                  "key": "Renewal_Date__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 12,
                  "y": 3
                },
                "displayOrder": 5
              },
              {
                "label": "Enterprise Age",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w27",
                "description": null,
                "config": {
                  "fieldName": "Enterprise_Age__gc",
                  "dataType": "NUMBER",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null,
                    "width": 4,
                    "rollup": null,
                    "requiredDisabled": false,
                    "SEARCH_CONTROLLER": {
                      "objectName": null,
                      "fields": []
                    }
                  },
                  "formatOptions": {
                    "type": "NUMBER",
                    "numericalSummarization": "None"
                  },
                  "scale": 0,
                  "fieldPath": null,
                  "key": "Enterprise_Age__gc",
                  "lookupDisplayField": null
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 8,
                  "y": 3
                },
                "displayOrder": 6
              },
              {
                "label": "Active Plan",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w50",
                "config": {
                  "fieldName": "Active_Plan__gc",
                  "dataType": "STRING",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": null,
                  "key": "Active_Plan__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 4,
                  "y": 3
                },
                "displayOrder": 7
              },
              {
                "label": "Plan State",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w51",
                "config": {
                  "fieldName": "Plan_State__gc",
                  "dataType": "PICKLIST",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": null,
                  "key": "Plan_State__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 0,
                  "y": 3
                },
                "displayOrder": 8
              },
              {
                "label": "Last 5 Timeline Activities",
                "widgetCategory": "Standard",
                "widgetType": "TIMELINE",
                "subType": "TIMELINE",
                "itemId": "w3",
                "className": "",
                "dimensionDetails": {
                  "rows": 9,
                  "cols": 8,
                  "maxItemCols": 12,
                  "maxItemRows": 12,
                  "minItemCols": 6,
                  "minItemRows": 6
                },
                "mini360Dimensions": {
                  "rows": 9,
                  "cols": 24
                },
                "axisDetails": {
                  "x": 16,
                  "y": 6
                },
                "displayOrder": 9
              },
              {
                "label": "Purchased Licenses",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w56",
                "description": null,
                "config": {
                  "fieldName": "Purchased_Licenses_SFDC__gc",
                  "dataType": "NUMBER",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null,
                    "width": 4,
                    "rollup": null,
                    "requiredDisabled": false,
                    "SEARCH_CONTROLLER": {
                      "objectName": null,
                      "fields": []
                    }
                  },
                  "formatOptions": {
                    "type": "NUMBER",
                    "numericalSummarization": "None"
                  },
                  "scale": 0,
                  "fieldPath": null,
                  "key": "Purchased_Licenses_SFDC__gc",
                  "lookupDisplayField": null
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 0,
                  "y": 6
                },
                "displayOrder": 10
              },
              {
                "label": "License Occupancy %",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w58",
                "description": null,
                "config": {
                  "fieldName": "License_Occupancy_SFDC__gc",
                  "dataType": "PERCENTAGE",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null,
                    "width": 4,
                    "rollup": null,
                    "requiredDisabled": false,
                    "SEARCH_CONTROLLER": {
                      "fields": []
                    }
                  },
                  "formatOptions": {
                    "type": null,
                    "numericalSummarization": null
                  },
                  "scale": 0,
                  "fieldPath": null,
                  "key": "License_Occupancy_SFDC__gc",
                  "lookupDisplayField": null
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 8,
                  "y": 6
                },
                "displayOrder": 11
              },
              {
                "label": "Used Licenses",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w59",
                "description": null,
                "config": {
                  "fieldName": "Used_Licenses_SFDC__gc",
                  "dataType": "NUMBER",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null,
                    "width": 4,
                    "rollup": null,
                    "requiredDisabled": false,
                    "SEARCH_CONTROLLER": {
                      "objectName": null,
                      "fields": []
                    }
                  },
                  "formatOptions": {
                    "type": "NUMBER",
                    "numericalSummarization": "None"
                  },
                  "scale": 0,
                  "fieldPath": null,
                  "key": "Used_Licenses_SFDC__gc",
                  "lookupDisplayField": null
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 4,
                  "y": 6
                },
                "displayOrder": 12
              },
              {
                "label": "MAU%",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w60",
                "config": {
                  "fieldName": "zzz__gc",
                  "dataType": "PERCENTAGE",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 1,
                  "fieldPath": null,
                  "key": "zzz__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 12,
                  "y": 6
                },
                "displayOrder": 13
              },
              {
                "label": "WCU% Last 4 Week Avg",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w61",
                "config": {
                  "fieldName": "WCU_Last_4_Week_Avg__gc",
                  "dataType": "PERCENTAGE",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 1,
                  "fieldPath": null,
                  "key": "WCU_Last_4_Week_Avg__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 0,
                  "y": 9
                },
                "displayOrder": 14
              },
              {
                "label": "SCIM Status",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w36",
                "config": {
                  "fieldName": "SCIM_Status__gc",
                  "dataType": "STRING",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": null,
                  "key": "SCIM_Status__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 8,
                  "y": 9
                },
                "displayOrder": 15
              },
              {
                "label": "SSO Status",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w35",
                "config": {
                  "fieldName": "SSO_Status__gc",
                  "dataType": "STRING",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": null,
                  "key": "SSO_Status__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 4,
                  "y": 9
                },
                "displayOrder": 16
              },
              {
                "label": "Domain Capture Status",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w34",
                "config": {
                  "fieldName": "Domain_Capture_Status__gc",
                  "dataType": "STRING",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": null,
                  "key": "Domain_Capture_Status__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 12,
                  "y": 9
                },
                "displayOrder": 17
              },
              {
                "label": "Domain Verification Status",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w37",
                "config": {
                  "fieldName": "Domain_Verification_Status__gc",
                  "dataType": "STRING",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": null,
                  "key": "Domain_Verification_Status__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 4,
                  "y": 12
                },
                "displayOrder": 18
              },
              {
                "label": "Key Domain",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w57",
                "config": {
                  "fieldName": "Key_Domain__gc",
                  "dataType": "STRING",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": null,
                  "key": "Key_Domain__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 8,
                  "y": 12
                },
                "displayOrder": 19
              },
              {
                "label": "User Group Status",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w38",
                "config": {
                  "fieldName": "User_Group_Status__gc",
                  "dataType": "STRING",
                  "objectName": "relationship",
                  "objectLabel": "Relationship",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": null,
                  "key": "User_Group_Status__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 0,
                  "y": 12
                },
                "displayOrder": 20
              },
              {
                "label": "Sub-Industry",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w68",
                "config": {
                  "fieldName": "SubIndustry__gc",
                  "dataType": "PICKLIST",
                  "objectName": "company",
                  "objectLabel": "Company",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": {
                    "lookupId": "CompanyId__gr",
                    "lookupName": "CompanyId__gr",
                    "legacyLookupId": "873d2b46-a974-4c88-a892-82ea0017b4aa",
                    "left": {
                      "type": "BASE_FIELD",
                      "fieldName": "Gsid",
                      "label": "GSID",
                      "objectName": "company",
                      "hasLookup": false,
                      "displayOrder": 0
                    },
                    "right": {
                      "type": "BASE_FIELD",
                      "fieldName": "CompanyId",
                      "label": "Company Id",
                      "objectName": "relationship",
                      "hasLookup": false,
                      "displayOrder": 0
                    },
                    "fieldPath": null
                  },
                  "key": "CompanyId_SubIndustry__gc"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 4,
                  "y": 15
                },
                "displayOrder": 21
              },
              {
                "label": "AI Cheat Sheet",
                "widgetCategory": "Standard",
                "widgetType": "CUSTOMER_HIGHLIGHTS",
                "subType": "CUSTOMER_HIGHLIGHTS",
                "itemId": "w29",
                "className": null,
                "dimensionDetails": {
                  "rows": 15,
                  "cols": 9,
                  "maxItemCols": 11,
                  "maxItemRows": 15,
                  "minItemCols": 9,
                  "minItemRows": 15
                },
                "mini360Dimensions": {
                  "rows": 12,
                  "cols": 24
                },
                "axisDetails": {
                  "x": 8,
                  "y": 15
                },
                "displayOrder": 22
              },
              {
                "label": "Industry (Dropdown)",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w67",
                "config": {
                  "fieldName": "IndustryNew",
                  "dataType": "MULTISELECTDROPDOWNLIST",
                  "objectName": "company",
                  "objectLabel": "Company",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": {
                    "lookupId": "CompanyId__gr",
                    "lookupName": "CompanyId__gr",
                    "legacyLookupId": "873d2b46-a974-4c88-a892-82ea0017b4aa",
                    "left": {
                      "type": "BASE_FIELD",
                      "fieldName": "Gsid",
                      "label": "GSID",
                      "objectName": "company",
                      "hasLookup": false,
                      "displayOrder": 0
                    },
                    "right": {
                      "type": "BASE_FIELD",
                      "fieldName": "CompanyId",
                      "label": "Company Id",
                      "objectName": "relationship",
                      "hasLookup": false,
                      "displayOrder": 0
                    },
                    "fieldPath": null
                  },
                  "key": "CompanyId_IndustryNew"
                },
                "dimensionDetails": {
                  "rows": 3,
                  "cols": 4,
                  "maxItemCols": 6,
                  "maxItemRows": 3,
                  "minItemCols": 3,
                  "minItemRows": 3
                },
                "mini360Dimensions": {
                  "rows": 3,
                  "cols": 6
                },
                "axisDetails": {
                  "x": 0,
                  "y": 15
                },
                "displayOrder": 23
              },
              {
                "label": "Installed Technologies",
                "widgetCategory": "Field",
                "widgetType": "CR",
                "subType": "FIELD",
                "itemId": "w69",
                "config": {
                  "fieldName": "Installed_Technologies__gc",
                  "dataType": "RICHTEXTAREA",
                  "objectName": "company",
                  "objectLabel": "Company",
                  "properties": {
                    "editable": false,
                    "required": null,
                    "navigationConfig": null
                  },
                  "formatOptions": {},
                  "scale": 0,
                  "fieldPath": {
                    "lookupId": "CompanyId__gr",
                    "lookupName": "CompanyId__gr",
                    "legacyLookupId": "873d2b46-a974-4c88-a892-82ea0017b4aa",
                    "left": {
                      "type": "BASE_FIELD",
                      "fieldName": "Gsid",
                      "label": "GSID",
                      "objectName": "company",
                      "hasLookup": false,
                      "displayOrder": 0
                    },
                    "right": {
                      "type": "BASE_FIELD",
                      "fieldName": "CompanyId",
                      "label": "Company Id",
                      "objectName": "relationship",
                      "hasLookup": false,
                      "displayOrder": 0
                    },
                    "fieldPath": null
                  },
                  "key": "CompanyId_Installed_Technologies__gc"
                },
                "dimensionDetails": {
                  "rows": 4,
                  "cols": 7,
                  "maxItemCols": 24,
                  "maxItemRows": 9,
                  "minItemCols": 6,
                  "minItemRows": 4
                },
                "mini360Dimensions": {
                  "rows": 8,
                  "cols": 24
                },
                "axisDetails": {
                  "x": 17,
                  "y": 15
                },
                "displayOrder": 24
              },
              {
                "label": "Team Current Add-on",
                "widgetCategory": "Report",
                "widgetType": "REPORT",
                "subType": "REPORT",
                "config": {
                  "reportName": "Team Current Add-on",
                  "reportId": "0a1fd97a-30fc-4f7d-a422-950dcd5058a9",
                  "reportTypes": [],
                  "sourceType": "MDA",
                  "visualizationType": "GRID",
                  "description": "",
                  "createdBy": "Andor Fuhrer",
                  "modifiedBy": "Andor Fuhrer",
                  "label": "Team Current Add-on",
                  "collectionDetail": {
                    "objectName": "product_usage_team_current_addons__gc",
                    "objectLabel": "Product Usage Team Current Addons",
                    "connectionId": "MDA",
                    "connectionType": "MDA",
                    "dataStoreType": "HAPOSTGRES",
                    "label": "Product Usage Team Current Addons",
                    "dragDisabled": true
                  }
                },
                "itemId": "w66",
                "dimensionDetails": {
                  "rows": 6,
                  "cols": 8,
                  "maxItemCols": 24,
                  "maxItemRows": 12,
                  "minItemCols": 8,
                  "minItemRows": 6
                },
                "mini360Dimensions": {
                  "rows": 9,
                  "cols": 24
                },
                "axisDetails": {
                  "x": 0,
                  "y": 18
                },
                "displayOrder": 25
              }
            ]
          },
          "sectionType": "SUMMARY",
          "configured": true,
          "scope": "GLOBAL",
          "displayOrder": 0
        },
        {
          "sectionId": "e675ed66-d573-42be-bcbe-24e0d3fa1506",
          "label": "Customer Goals",
          "sectionType": "CUSTOMER_GOALS",
          "configured": true,
          "scope": "LOCAL",
          "displayOrder": 1
        },
        {
          "sectionId": "335146c4-db53-4dc5-bb1d-b9b8c723afa2",
          "label": "Scorecard",
          "sectionType": "SCORECARD",
          "configured": true,
          "scope": "LOCAL",
          "displayOrder": 2
        },
        "... [truncated]"
      ],
      "default": true
    },
    "resolutionBean": null,
    "data": {
      "relationship_CompanyId__gr.Name": "7 - Eleven (7 - 11) Inc.",
      "relationship_CompanyId__gr.SfdcAccountId": "0011K00002H9qNmQAJ",
      "relationship_Status_PicklistLabel_SystemName": null,
      "relationship_Status_PicklistLabel": null,
      "relationship_Name": "7-Eleven - Ent 68639 - 10/1 - 68639"
    },
    "quickActions": [
      {
        "sectionType": "TIMELINE",
        "label": "Activity"
      },
      {
        "sectionType": "COCKPIT",
        "label": "CTA"
      },
      {
        "sectionType": "PERSON",
        "label": "Person"
      }
    ]
  },
  "message": null,
  "localizedMessage": null
}
```


### POST /v2/galaxy/cr360/data/section
**URL:** `https://postman.us2.gainsightcloud.com/v2/galaxy/cr360/data/section`
**Status Codes:** [200]
**Response Size:** 4571 - 4571 bytes

**Request Body:**
```json
{
  "entityId": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU",
  "layoutId": "84441ebc-079b-4e3d-85f0-d0010b5a3eed",
  "sectionId": "5ea4671a-3727-46ee-a1dc-797989a65f97",
  "scope": "LOCAL",
  "objectName": "relationship",
  "entityTypeId": "1P06GH6NC0YFN2YXBDUFZX2FIJADKKBZ3AD3"
}
```

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "b340fc5c-e606-4ea0-9d46-5c521eb0fc74",
  "data": [
    {
      "w60": {
        "k": 81.0,
        "v": "81.0",
        "p": null,
        "s": "%",
        "fv": "81.0 %",
        "sn": null,
        "im": false
      },
      "w51": {
        "k": "1I00T2LKNPZNZY3IWX7ANUQ1OC8FQG2N5VVM",
        "v": "1I00T2LKNPZNZY3IWX7ANUQ1OC8FQG2N5VVM",
        "p": null,
        "s": null,
        "fv": "Active",
        "sn": "Active",
        "im": false
      },
      "w50": {
        "k": "Enterprise (2021)",
        "v": "Enterprise (2021)",
        "p": null,
        "s": null,
        "fv": "Enterprise (2021)",
        "sn": null,
        "im": false
      },
      "w61": {
        "k": 34.7,
        "v": "34.7",
        "p": null,
        "s": "%",
        "fv": "34.7 %",
        "sn": null,
        "im": false
      },
      "w30": {
        "k": "images/3a5744bf-4121-4e2e-ab6f-ee7bf36fc524/1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU/v0/original.jpg",
        "v": "https://prod-images.us2.gainsightcloud.com/images/3a5744bf-4121-4e2e-ab6f-ee7bf36fc524/1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU/v0/original.jpg?Policy=eyJTdGF0ZW1lbnQiOiBbeyJSZXNvdXJjZSI6Imh0dHBzOi8vcHJvZC1pbWFnZXMudXMyLmdhaW5zaWdodGNsb3VkLmNvbS9pbWFnZXMvM2E1NzQ0YmYtNDEyMS00ZTJlLWFiNmYtZWU3YmYzNmZjNTI0LzFQMDVWWlZHWjUyNkc3UVZOV1lFRFNUVUdFMTRKTUkwS0RJVS92MC9vcmlnaW5hbC5qcGciLCJDb25kaXRpb24iOnsiRGF0ZUxlc3NUaGFuIjp7IkFXUzpFcG9jaFRpbWUiOjE3NTgxMDA3ODl9LCJEYXRlR3JlYXRlclRoYW4iOnsiQVdTOkVwb2NoVGltZSI6MTc1ODAxNDMyOX19fV19&Signature=WH3fzkkDvXpJJ56AueEooBSnQ~WeCpLylRKRVpvMK0FHAX2kleauTsbQ9ML9R2Cmffu9saVzHVHijiIwvdV95D2~wPLhTpKN9TAuZ0chTYHe5aNF0NJX5JFYgDoa0Uw0P3NndY90725cuaFnasCFX3BV5fcVUr2M60WOF5LygaVLG6jeycFgND0h2ucaQ0agWC4udVbnIbq9sgXTgmt5Dev5Rz9XM2~UAELEsGT5DJkZBs~QfmpW~aUjcEjGCbjlIUHHEnIrCvTTDGZKcs4EOkw-6snzIWAYslr3HIVdtOX4rowE1bVlVhVKQHYK9f5oQIpujtsxB3SAby6ya0FqHw__&Key-Pair-Id=APKAST4MNX7I7PA4XUMP",
        "p": null,
        "s": null,
        "fv": "https://prod-images.us2.gainsightcloud.com/images/3a5744bf-4121-4e2e-ab6f-ee7bf36fc524/1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU/v0/original.jpg?Policy=eyJTdGF0ZW1lbnQiOiBbeyJSZXNvdXJjZSI6Imh0dHBzOi8vcHJvZC1pbWFnZXMudXMyLmdhaW5zaWdodGNsb3VkLmNvbS9pbWFnZXMvM2E1NzQ0YmYtNDEyMS00ZTJlLWFiNmYtZWU3YmYzNmZjNTI0LzFQMDVWWlZHWjUyNkc3UVZOV1lFRFNUVUdFMTRKTUkwS0RJVS92MC9vcmlnaW5hbC5qcGciLCJDb25kaXRpb24iOnsiRGF0ZUxlc3NUaGFuIjp7IkFXUzpFcG9jaFRpbWUiOjE3NTgxMDA3ODl9LCJEYXRlR3JlYXRlclRoYW4iOnsiQVdTOkVwb2NoVGltZSI6MTc1ODAxNDMyOX19fV19&Signature=WH3fzkkDvXpJJ56AueEooBSnQ~WeCpLylRKRVpvMK0FHAX2kleauTsbQ9ML9R2Cmffu9saVzHVHijiIwvdV95D2~wPLhTpKN9TAuZ0chTYHe5aNF0NJX5JFYgDoa0Uw0P3NndY90725cuaFnasCFX3BV5fcVUr2M60WOF5LygaVLG6jeycFgND0h2ucaQ0agWC4udVbnIbq9sgXTgmt5Dev5Rz9XM2~UAELEsGT5DJkZBs~QfmpW~aUjcEjGCbjlIUHHEnIrCvTTDGZKcs4EOkw-6snzIWAYslr3HIVdtOX4rowE1bVlVhVKQHYK9f5oQIpujtsxB3SAby6ya0FqHw__&Key-Pair-Id=APKAST4MNX7I7PA4XUMP",
        "sn": null,
        "im": false
      },
      "w65": {
        "fv": "Adrian Nardella",
        "im": false,
        "v": "Adrian Nardella",
        "gsid": "1P01Q1IO46XBY7CLHNJ3NDXQM5RN9JZSSWD2",
        "k": "Adrian Nardella"
      },
      "w24": {
        "k": "2025-09-30",
        "v": "9/30/2025",
        "p": null,
        "s": null,
        "fv": "9/30/2025",
        "sn": null,
        "im": false
      },
      "w35": {
        "k": "Enabled",
        "v": "Enabled",
        "p": null,
        "s": null,
        "fv": "Enabled",
        "sn": null,
        "im": false
      },
      "w57": {
        "fv": "",
        "im": false
      },
      "w68": {
        "k": "1I00GUGERSLRWJ1MWC664SRP3JZJV6BSWOA8",
        "v": "1I00GUGERSLRWJ1MWC664SRP3JZJV6BSWOA8",
        "p": null,
        "s": null,
        "fv": "Convenience Stores, Gas Stations & Liquor Stores",
        "sn": "Convenience Stores, Gas Stations & Liquor Stores",
        "im": false
      },
      "w56": {
        "k": 445,
        "v": "445",
        "p": null,
        "s": "",
        "fv": "445",
        "sn": null,
        "im": false
      },
      "w34": {
        "k": "Domain Capture Enabled",
        "v": "Domain Capture Enabled",
        "p": null,
        "s": null,
        "fv": "Domain Capture Enabled",
        "sn": null,
        "im": false
      },
      "w67": {
        "k": "1I006JHHQUNKT82B8F0L4AEJCP7MF1UT2RV5",
        "v": "1I006JHHQUNKT82B8F0L4AEJCP7MF1UT2RV5",
        "p": null,
        "s": null,
        "fv": "Retail",
        "sn": "Retail",
        "im": false
      },
      "w59": {
        "k": 414,
        "v": "414",
        "p": null,
        "s": "",
        "fv": "414",
        "sn": null,
        "im": false
      },
      "w37": {
        "k": "DV Enabled",
        "v": "DV Enabled",
        "p": null,
        "s": null,
        "fv": "DV Enabled",
        "sn": null,
        "im": false
      },
      "w58": {
        "k": 93,
        "v": "93",
        "p": null,
        "s": "%",
        "fv": "93 %",
        "sn": null,
        "im": false
      },
      "w36": {
        "k": "Enabled",
        "v": "Enabled",
        "p": null,
        "s": null,
        "fv": "Enabled",
        "sn": null,
        "im": false
      },
      "w69": {
        "k": "Swagger, Github, Slack, API Gateway, soapUI, Apigee API Platform, Confluence, Jira",
        "v": "Swagger, Github, Slack, API Gateway, soapUI, Apigee API Platform, Confluence, Jira",
        "p": null,
        "s": null,
        "fv": "Swagger, Github, Slack, API Gateway, soapUI, Apigee API Platform, Confluence, Jira",
        "sn": null,
        "im": false
      },
      "w17": {
        "fv": "Sean Reed",
        "im": false,
        "v": "Sean Reed",
        "gsid": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F",
        "k": "1P01Q1IO46XBY7CLHNHF9CBZ88QCJ5E0VV0F"
      },
      "w16": {
        "k": 465221,
        "v": "465,221",
        "p": "USD",
        "s": "",
        "fv": "USD 465,221",
        "sn": null,
        "im": false
      },
      "w27": {
        "k": 1081,
        "v": "1,081",
        "p": null,
        "s": "",
        "fv": "1,081",
        "sn": null,
        "im": false
      },
      "w38": {
        "k": "Enabled",
        "v": "Enabled",
        "p": null,
        "s": null,
        "fv": "Enabled",
        "sn": null,
        "im": false
      }
    }
  ],
  "message": null,
  "localizedMessage": null
}
```


### POST /v2/galaxy/spaces/assignment/resolve/cid
**URL:** `https://postman.us2.gainsightcloud.com/v2/galaxy/spaces/assignment/resolve/cid?includeSectionConfig=false`
**Query Parameters:** includeSectionConfig
**Status Codes:** [401]
**Response Size:** 337 - 337 bytes

**Request Body:**
```json
{
  "entityType": "Relationship",
  "entityId": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU",
  "sharingType": "external",
  "relationshipId": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU"
}
```

**Response:**
```json
{
  "result": false,
  "errorCode": "GS_2402",
  "errorDesc": "Not authorized to perform operation.",
  "localizedErrorDesc": "Not authorized to perform operation.",
  "requestId": "25815dbc-cfe0-4f24-83c8-05b28604cb4a",
  "data": null,
  "message": "Don't have required permissions. Authorization failed.",
  "localizedMessage": "Not authorized to perform operation."
}
```


### POST /v2/galaxy/transform/filter/relationship
**URL:** `https://postman.us2.gainsightcloud.com/v2/galaxy/transform/filter/relationship`
**Status Codes:** [200]
**Response Size:** 702 - 702 bytes

**Request Body:**
```json
{
  "relationshipId": "1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU",
  "relationshipTypeId": "1P06GH6NC0YFN2YXBDUFZX2FIJADKKBZ3AD3",
  "objectName": "product_usage_team_metrics_weekly__gc",
  "objectLabel": "Product Usage Team Metrics Weekly"
}
```

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "62c1f2b9-6349-4921-a30f-b26b18c02bf9",
  "data": {
    "conditions": [
      {
        "leftOperand": {
          "type": "BASE_FIELD",
          "fieldName": "Postman_Team_ID__gc",
          "dbName": "postman_team_id__gc",
          "label": "Postman Team ID",
          "dataType": "LOOKUP",
          "objectName": "product_usage_team_metrics_weekly__gc",
          "objectDBName": "productusageteammetr_k94p65shd23a5744bf41214e2eab6fee7bf36fc524",
          "objectLabel": "Product Usage Team Metrics Weekly",
          "key": "Postman_Team_ID__gc"
        },
        "filterAlias": "bm_0",
        "logicalOperator": "AND",
        "comparisonOperator": "EQ",
        "rightOperandType": "VALUE",
        "filterValue": {
          "value": [
            "68639"
          ]
        }
      }
    ],
    "expression": "bm_0"
  },
  "message": null,
  "localizedMessage": null
}
```


---

## Configuration & Settings

### GET /v1/features/evaluate/GEN_AI_FOR_EMAILS
**URL:** `https://postman.us2.gainsightcloud.com/v1/features/evaluate/GEN_AI_FOR_EMAILS`
**Status Codes:** [200]
**Response Size:** 519 - 519 bytes

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "64f77185-276a-4b79-b6e3-568891644f19",
  "data": {
    "createdBy": "",
    "modifiedBy": "",
    "createdDate": 1713586566849,
    "modifiedDate": 1713586566849,
    "id": "true_variation_f0b16f4ba53c46db9a2c0c83d9f92c51",
    "featureId": "gen_ai_for_emails_c123c511dc934127938123f073acdc4d",
    "name": "TRUE VARIATION",
    "type": "BOOLEAN",
    "description": "The feature is enabled.",
    "value": true,
    "useAsDefault": false,
    "useAsInactive": false
  },
  "message": null,
  "localizedMessage": null
}
```


### GET /v1/features/evaluate/SS_HA_ENABLEMENT
**URL:** `https://postman.us2.gainsightcloud.com/v1/features/evaluate/SS_HA_ENABLEMENT`
**Status Codes:** [200]
**Response Size:** 480 - 480 bytes

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "b2522545-0678-4ac5-b49a-09338477f985",
  "data": {
    "createdBy": "",
    "modifiedBy": "",
    "createdDate": 1659335897599,
    "modifiedDate": 1659335897599,
    "id": "enabled_39d69e78d8d742718156691967906675",
    "featureId": "ss_ha_enablement_425df82622344a97b02572c8798fc2a4",
    "name": "ENABLED",
    "type": "MULTI_VARIATE",
    "description": "To enabled Success Snapshopt HA",
    "value": "ENABLED"
  },
  "message": null,
  "localizedMessage": null
}
```


### GET /v1/features/evaluation/EFA_ORG_WIDE_ADDRESS_ENABLEMENT/{id}a5744bf-4121-4e2e-ab6f-ee7bf36fc524
**URL:** `https://postman.us2.gainsightcloud.com/v1/features/evaluation/EFA_ORG_WIDE_ADDRESS_ENABLEMENT/3a5744bf-4121-4e2e-ab6f-ee7bf36fc524`
**Status Codes:** [200]
**Response Size:** 498 - 498 bytes

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "18c2833e-11e6-4912-a2d8-dd963b853b83",
  "data": {
    "createdBy": "",
    "modifiedBy": "",
    "createdDate": 1637054624483,
    "modifiedDate": 1637054624483,
    "id": "true_variation_c94a7186cbba4772b5852b53a40dbf05",
    "featureId": "efa_org_wide_address_enablement_f8d5fe65e9c846ec8b6f4cd32deb96fe",
    "name": "TRUE_VARIATION",
    "type": "BOOLEAN",
    "description": "From Address feature is enabled",
    "value": true
  },
  "message": null,
  "localizedMessage": null
}
```


### GET /v1/gsassist/config/admin/email/
**URL:** `https://postman.us2.gainsightcloud.com/v1/gsassist/config/admin/email/?source=GS_HOME`
**Query Parameters:** source
**Status Codes:** [200]
**Response Size:** 812 - 812 bytes

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "77993751-f8fc-4363-b92e-510fd0f6d5a5",
  "data": [
    {
      "createdDate": null,
      "source": "GS_HOME",
      "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
      "config": {
        "allowSendingEmails": false,
        "modifyDefaultTokenMapping": false,
        "automaticallyLogEmailsInTimeline": false,
        "chooseSenderEmailAddress": false
      },
      "emailLimits": {
        "maxPersonsInPersonGroup": 500,
        "maxRecipientsInEmail": 2000,
        "maxEmailsToSendInDay": 4000,
        "maxEmailsToSendInMonth": 4000,
        "maxAllowedPersonsInPersonGroup": 2000,
        "maxAllowedRecipientsInEmail": 2000,
        "maxAllowedEmailsToSendInDay": 10000,
        "maxAllowedEmailsToSendInMonth": 10000
      },
      "createdBy": null,
      "createdByName": null,
      "modifiedBy": null,
      "modifiedByName": null,
      "modifiedDateStr": null,
      "createdDateStr": null
    }
  ],
  "message": null,
  "localizedMessage": null
}
```


### GET /v1/layouts/state/search
**URL:** `https://postman.us2.gainsightcloud.com/v1/layouts/state/search?[query_params]`
**Query Parameters:** moduleName, referenceId
**Status Codes:** [200]
**Response Size:** 102 - 102 bytes

**Response:**
```json
{
  "result": true,
  "requestId": "6727c4dc-fbc8-4856-957e-1d1afd1cdddf",
  "data": [],
  "duration": 57,
  "alerts": []
}
```


### GET /v2/exports/google/fetch/user
**URL:** `https://postman.us2.gainsightcloud.com/v2/exports/google/fetch/user`
**Status Codes:** [200]
**Response Size:** 207 - 207 bytes

**Response:**
```json
{
  "result": false,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "f2a5974e-4b91-4f10-84bc-3a3fd3002c3c",
  "data": "User details cannot be fetched.",
  "message": null,
  "localizedMessage": null
}
```


### GET /v2/exports/ss
**URL:** `https://postman.us2.gainsightcloud.com/v2/exports/ss`
**Status Codes:** [200]
**Response Size:** 175 - 175 bytes

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "6491bbf3-fa5b-4f00-a115-81b285d2d8f3",
  "data": [],
  "message": null,
  "localizedMessage": null
}
```


### GET /v2/externalsharing/share/extlayouts/layouts/R360
**URL:** `https://postman.us2.gainsightcloud.com/v2/externalsharing/share/extlayouts/layouts/R360`
**Status Codes:** [200]
**Response Size:** 175 - 175 bytes

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "400f1720-c43b-4b12-bbb8-304a63a56daf",
  "data": [],
  "message": null,
  "localizedMessage": null
}
```


---

## Other APIs

### GET /v1/api/emailConfig/compliance
**URL:** `https://postman.us2.gainsightcloud.com/v1/api/emailConfig/compliance`
**Status Codes:** [200]
**Response Size:** 688 - 688 bytes

**Response:**
```json
{
  "result": true,
  "requestId": "85b7165c-637b-4b9e-ab29-a603aea7d3aa",
  "data": [
    {
      "createdDate": 1748895478096,
      "createdBy": "1P01Q1IO46XBY7CLHNYNQ1FFYQHWCYZAWP44",
      "createdByName": "Emma Johnson-Prabhakar",
      "modifiedBy": "1P01Q1IO46XBY7CLHNC17YPABP4GTGTRIGP2",
      "modifiedByName": "Ben Wanless",
      "tenantId": "3a5744bf-4121-4e2e-ab6f-ee7bf36fc524",
      "emailFeatureType": "COCKPIT",
      "emailAddresses": [],
      "eMailAssistDomains": [
        "postman.com"
      ],
      "defaultEmailAssistDomain": "postman.com",
      "defaultEmailAssistFromDomain": "postman.com",
      "defaultEmailAssistReplyDomain": "postman.com",
      "fromDomainLocked": false,
      "replyToDomainLocked": false,
      "createdDateStr": "2025-06-02 20:17:58 UTC",
      "modifiedDateStr": "2025-06-03 20:21:02 UTC"
    }
  ]
}
```


### GET /v1/communications/email/serviceProvider
**URL:** `https://postman.us2.gainsightcloud.com/v1/communications/email/serviceProvider`
**Status Codes:** [200]
**Response Size:** 183 - 183 bytes

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "5cd49c1a-1882-4795-ad76-7eb572cb982a",
  "data": "SENDGRID",
  "message": null,
  "localizedMessage": null
}
```


### GET /v1/gsassist/fromaddress/my-approved
**URL:** `https://postman.us2.gainsightcloud.com/v1/gsassist/fromaddress/my-approved`
**Status Codes:** [200]
**Response Size:** 195 - 195 bytes

**Response:**
```json
{
  "result": true,
  "errorCode": null,
  "errorDesc": null,
  "localizedErrorDesc": null,
  "requestId": "db805817-9851-4dc8-8aa2-194dab75e77d",
  "data": {
    "fromAddressList": []
  },
  "message": null,
  "localizedMessage": null
}
```


### GET /v1/messenger/entity/{id}
**URL:** `https://postman.us2.gainsightcloud.com/v1/messenger/entity/1P05VZVGZ526G7QVNWYEDSTUGE14JMI0KDIU`
**Status Codes:** [200]
**Response Size:** 34 - 34 bytes

**Response:**
```json
{
  "data": {
    "id": null
  },
  "result": true
}
```


### GET /v1/messenger/token
**URL:** `https://postman.us2.gainsightcloud.com/v1/messenger/token`
**Status Codes:** [200]
**Response Size:** 5548 - 5548 bytes

**Response:**
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJ1c2VySWQiOiIxUDAxUTFJTzQ2WEJZN0NMSE4yVUJUODkwTzE3OTBCNDNOVkEiLCJ0ZW5hbnRJZCI6IjNhNTc0NGJmLTQxMjEtNGUyZS1hYjZmLWVlN2JmMzZmYzUyNCIsInRlbmFudFRpbWVab25lIjoiQW1lcml... [truncated]"
}
```


### GET /v1/spaces/enabled/consumption
**URL:** `https://postman.us2.gainsightcloud.com/v1/spaces/enabled/consumption`
**Status Codes:** [403]
**Response Size:** 269 - 269 bytes

**Response:**
```json
{
  "result": false,
  "errorCode": "GS_2402",
  "errorDesc": "Don't have required permissions. Authorization failed.",
  "localizedErrorDesc": "Not authorized to perform operation.",
  "requestId": "2cf29aaa-016a-4cc9-9af4-c3eabb07942c",
  "data": null,
  "message": null,
  "localizedMessage": null
}
```


---

## Summary
- **Total API Endpoints:** 31
- **Total API Calls Analyzed:** 32
- **HTTP Methods:** {'GET': 23, 'POST': 8}
- **Status Codes Found:** [200, 401, 403]
