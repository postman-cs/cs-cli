# Gainsight HAR Analysis Report
## GraphQL and Schema Discovery Findings

**Analysis Date:** July 1, 2025  
**HAR File:** `/Users/jared.boynton@postman.com/_csa_deprecated/app_backend/platforms/gainsight/discovery/gainsight.har`  
**Total HTTP Entries Analyzed:** 1,916

---

## Executive Summary

The Gainsight HAR analysis reveals that **Gainsight does NOT use traditional GraphQL** endpoints. Instead, it employs a sophisticated **proprietary query and metadata API system** with extensive schema discovery capabilities. The platform provides rich metadata APIs that give comprehensive access to its data model and schema information.

## Key Findings

### ❌ GraphQL Status
- **No traditional GraphQL endpoints found** (`/graphql`, `/gql`)
- **No GraphQL introspection queries** (`__schema`, `__type`)
- **No GraphQL operations** (query/mutation/subscription)

### ✅ Query System Discovery
- **43 BI (Business Intelligence) query endpoints** discovered using `/v3/bi/query/` pattern
- **Custom query language** with SQL-like structure for data extraction
- **Report-based query system** with sophisticated filtering and aggregation

### 🔍 Schema Discovery Goldmine

The analysis uncovered **143 metadata/schema endpoints** that provide comprehensive access to Gainsight's data model:

#### Primary Schema Discovery Endpoints

1. **Describe Endpoints** (82 found)
   - `/v1/api/describe/{connection}/{object}` - Complete object schema with fields, types, relationships
   - `/v3/bi/reporting/describe/{connection}/{object}` - Reporting-specific schema information
   - `/v1/api/reporting/describe/fields` - Field metadata with data types

2. **Bootstrap Endpoints** (20 found)
   - `/v1/users/meta/bootstrap` - User and permission metadata
   - `/v3/bi/reporting-ui/bootstrap` - Reporting system configuration
   - `/v2/galaxy/bootstrap/summaryRibbon/config` - UI component schema

3. **Configuration Endpoints** (34 found)
   - Various `/config/` endpoints for system metadata
   - Admin configuration endpoints
   - Feature enablement configs

4. **List Endpoints** (2 found)
   - `/v3/bi/reporting/describe/listsources` - Available data sources
   - `/v3/bi/reporting/describe/listobjects/{connection}` - Available objects per connection

---

## Comprehensive Schema Information Discovered

### Core Gainsight Objects (36 unique)
- **Call To Action** - Customer success tasks and actions
- **Company** - Customer/account records  
- **Relationship** - Customer relationships and hierarchies
- **CS Task** - Customer success team tasks
- **Person** - Individual contacts
- **Scorecard Master** - Health scoring definitions
- **Playbook** - Automated workflow definitions
- **Activity Timeline** - Customer interaction history

### Field Metadata (841+ unique fields)
Sample critical fields discovered:
- Customer identifiers (Gsid, CompanyId, RelationshipId)
- Financial metrics (Arr, Mrr, Revenue)
- Health indicators (CurrentScore, HealthStatus, Csat, Nps)
- Engagement data (LastTouchDate, Sentiment, Stage)
- User assignment (Csm, AccountOwner, PreviousCsm)

### Data Types (19 types)
- **GSID** - Gainsight unique identifier
- **LOOKUP** - Foreign key relationships
- **CURRENCY** - Financial values
- **PICKLIST** - Dropdown/enum values
- **MULTISELECTDROPDOWNLIST** - Multi-select enums
- **DATETIME/DATE** - Temporal data
- **RICHTEXTAREA** - Rich text content
- **BOOLEAN, STRING, NUMBER** - Standard types

### Connection/Data Sources
- **MDA** - Primary Gainsight data store (appears to be PostgreSQL-based)
- **HAPOSTGRES** - High Availability PostgreSQL cluster
- **REDSHIFT** - Data warehouse for analytics
- **UNIVERSAL_DATA_SET** - Aggregated/computed datasets

---

## Most Valuable Discovery Endpoints

### 1. Complete Object Schema Endpoints
```
GET /v1/api/describe/mda/cs_task?ci=mda&ic=true&cl=1&ppos=true&hcl=true&pasd=true&op=META&ade=true
- 674 fields discovered
- 18 related objects
- 910KB response with complete schema

GET /v1/api/describe/mda/call_to_action?ci=mda&ic=true&cl=1&ppos=true&hcl=true&pasd=true&op=META&ade=true
- 662 fields discovered  
- 16 related objects
- 883KB response with full metadata
```

### 2. Field-Level Metadata
```
POST /v3/bi/reporting/describe/fields
- Detailed field definitions with data types
- Object relationships and constraints
- Validation rules and picklist values
```

### 3. Data Source Discovery
```
GET /v3/bi/reporting/describe/listsources
GET /v3/bi/reporting/describe/listobjects/MDA
- Available connections and objects
- Permission-based access controls
```

---

## Query System Analysis

### BI Query Structure
Gainsight uses a sophisticated **report-based query system** instead of GraphQL:

```json
{
  "reportMaster": {
    "sourceDetails": {
      "objectName": "company",
      "connectionId": "MDA", 
      "dataStoreType": "HAPOSTGRES"
    },
    "showFields": [...],
    "whereFilters": {...},
    "orderByFields": [...],
    "pageSize": 50
  }
}
```

### Key Query Endpoints
- `/v3/bi/query/fetch-data` - Execute queries for data
- `/v3/bi/query/fetch-data-count` - Get record counts
- `/v3/bi/query/drilldown/fetch-data` - Navigate hierarchical data

---

## API Integration Recommendations

### Immediate Implementation Priorities

1. **Object Schema Discovery**
   - Use `/v1/api/describe/mda/{object}` for complete object schemas
   - Cache schema responses (responses are 300KB-900KB)
   - Essential objects: `company`, `relationship`, `call_to_action`, `cs_task`

2. **Field Metadata Discovery**  
   - Use `/v3/bi/reporting/describe/fields` for field-level details
   - Crucial for understanding data types and constraints

3. **Data Source Mapping**
   - Call `/v3/bi/reporting/describe/listsources` for available connections
   - Use `/v3/bi/reporting/describe/listobjects/MDA` for object lists

4. **Query System Integration**
   - Implement BI query format for data extraction
   - Support pagination and filtering via `whereFilters`
   - Use connection-aware queries with `connectionId=MDA`

### Authentication Context
All discovery endpoints require:
- Valid Gainsight session authentication
- Proper connection permissions (`ci=MDA`)
- Operation-level access (`op=META` for schema, `op=READ` for data)

---

## No Traditional GraphQL, But Rich Metadata APIs

**Conclusion:** While Gainsight doesn't use GraphQL, it provides superior schema discovery capabilities through its proprietary metadata API system. The platform offers:

- **Complete object schemas** with field definitions, types, and relationships
- **Real-time field metadata** with validation rules and constraints  
- **Dynamic object discovery** based on user permissions
- **Multi-connection support** with different data stores
- **Rich query capabilities** through the BI reporting system

This metadata-rich approach provides **better schema discovery than many GraphQL implementations** and enables building comprehensive data integration tools.